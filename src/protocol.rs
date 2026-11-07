use crate::config::Config;
use serde_json::json;
use std::{error::Error, io};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn handle_connection(
    mut stream: TcpStream,
    config: &Config,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let handshake = read_packet(&mut stream).await?;
    let mut cursor = 0;
    let packet_id = read_varint(&handshake, &mut cursor)?;
    if packet_id != 0 {
        return Err("expected handshake packet".into());
    }
    let _protocol_version = read_varint(&handshake, &mut cursor)?;
    let host_len = read_varint(&handshake, &mut cursor)?;
    if !(0..=32767).contains(&host_len) {
        return Err("invalid host length".into());
    }
    cursor = cursor
        .checked_add(host_len as usize)
        .ok_or("invalid host length")?;
    if cursor + 3 > handshake.len() {
        return Err("truncated handshake".into());
    }
    cursor += 2; // virtual host port
    let next_state = read_varint(&handshake, &mut cursor)?;

    match next_state {
        1 => status_exchange(&mut stream, config).await?,
        2 => send_disconnect(&mut stream, "Lithum is an experimental prototype: player login and world simulation are not implemented yet.").await?,
        _ => return Err("unsupported next state".into()),
    }
    Ok(())
}

async fn status_exchange(
    stream: &mut TcpStream,
    config: &Config,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let request = read_packet(stream).await?;
    let mut cursor = 0;
    if read_varint(&request, &mut cursor)? != 0 {
        return Err("expected status request".into());
    }
    let body = serde_json::to_vec(&json!({
        "version": { "name": "Lithum experimental", "protocol": 0 },
        "players": { "max": config.server.max_players, "online": 0 },
        "description": { "text": config.server.motd }
    }))?;
    let mut response = Vec::with_capacity(body.len() + 6);
    write_varint(0, &mut response);
    write_varint(body.len() as i32, &mut response);
    response.extend_from_slice(&body);
    write_packet(stream, &response).await?;

    let ping = read_packet(stream).await?;
    let mut cursor = 0;
    if read_varint(&ping, &mut cursor)? != 1 || ping.len() != cursor + 8 {
        return Err("expected ping packet".into());
    }
    let mut pong = Vec::with_capacity(9);
    write_varint(1, &mut pong);
    pong.extend_from_slice(&ping[cursor..]);
    write_packet(stream, &pong).await?;
    Ok(())
}

async fn send_disconnect(stream: &mut TcpStream, message: &str) -> io::Result<()> {
    let json = serde_json::to_string(&json!({ "text": message })).unwrap_or_else(|_| "{}".into());
    let mut payload = Vec::new();
    write_varint(0, &mut payload);
    write_varint(json.len() as i32, &mut payload);
    payload.extend_from_slice(json.as_bytes());
    write_packet(stream, &payload).await
}

async fn read_packet(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let length = read_stream_varint(stream).await? as usize;
    if length == 0 || length > 2 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "packet length out of bounds",
        ));
    }
    let mut packet = vec![0; length];
    stream.read_exact(&mut packet).await?;
    Ok(packet)
}

async fn write_packet(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    let mut frame = Vec::with_capacity(payload.len() + 5);
    write_varint(payload.len() as i32, &mut frame);
    frame.extend_from_slice(payload);
    stream.write_all(&frame).await
}

async fn read_stream_varint(stream: &mut TcpStream) -> io::Result<i32> {
    let mut value = 0i32;
    for i in 0..5 {
        let byte = stream.read_u8().await?;
        value |= ((byte & 0x7f) as i32) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "VarInt exceeds five bytes",
    ))
}

fn read_varint(bytes: &[u8], cursor: &mut usize) -> io::Result<i32> {
    let mut value = 0i32;
    for i in 0..5 {
        let byte = *bytes
            .get(*cursor)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "truncated VarInt"))?;
        *cursor += 1;
        value |= ((byte & 0x7f) as i32) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "VarInt exceeds five bytes",
    ))
}

fn write_varint(mut value: i32, out: &mut Vec<u8>) {
    loop {
        if value & !0x7f == 0 {
            out.push(value as u8);
            return;
        }
        out.push((value as u8 & 0x7f) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}
