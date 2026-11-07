mod config;
mod plugins;
mod protocol;

use config::Config;
use plugins::PluginManager;
use std::{error::Error, sync::Arc, time::Instant};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    sync::{Semaphore, watch},
    time::{self, MissedTickBehavior},
};

#[derive(Clone, Debug, Default)]
struct Metrics {
    tps: f64,
    memory_mb: u64,
    connections: usize,
    ticks: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load_or_create()?;
    let threads = if config.server.worker_threads == 0 {
        std::thread::available_parallelism().map_or(2, usize::from)
    } else {
        config.server.worker_threads
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()?;
    if let Err(error) = runtime.block_on(run(config)) {
        return Err(std::io::Error::other(error.to_string()).into());
    }
    Ok(())
}

async fn run(config: Config) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!(
        "Lithum Server {} — experimental protocol/status foundation",
        env!("CARGO_PKG_VERSION")
    );

    let plugins =
        PluginManager::load("plugins").map_err(|error| std::io::Error::other(error.to_string()))?;
    plugins.enable_all();

    let listener = TcpListener::bind((config.server.address.as_str(), config.server.port)).await?;
    let local_addr = listener.local_addr()?;
    println!("Listening on {local_addr}");
    println!("Console commands: status, help, stop");

    let (stop_tx, stop_rx) = watch::channel(false);
    let (metrics_tx, metrics_rx) = watch::channel(Metrics::default());
    let connection_slots = Arc::new(Semaphore::new(config.server.max_connections));

    let accept_task = tokio::spawn(accept_loop(
        listener,
        config.clone(),
        connection_slots.clone(),
        stop_rx.clone(),
    ));
    let metrics_task = tokio::spawn(metrics_loop(
        config.metrics.refresh_interval_ms,
        config.server.max_connections,
        connection_slots,
        metrics_tx,
        stop_rx.clone(),
        plugins.clone(),
    ));
    let console_task = tokio::spawn(console_loop(
        metrics_rx,
        plugins,
        stop_tx.clone(),
        stop_rx.clone(),
    ));

    let mut stop_wait = stop_rx;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => { println!("\nStopping Lithum..."); }
        _ = stop_wait.changed() => { println!("Stopping Lithum..."); }
        result = accept_task => {
            result.map_err(std::io::Error::other)??;
        }
        result = metrics_task => {
            result.map_err(std::io::Error::other)??;
        }
        result = console_task => {
            result.map_err(std::io::Error::other)??;
        }
    }
    let _ = stop_tx.send(true);
    Ok(())
}

async fn accept_loop(
    listener: TcpListener,
    config: Config,
    slots: Arc<Semaphore>,
    mut stop: watch::Receiver<bool>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        tokio::select! {
            _ = stop.changed() => if *stop.borrow() { return Ok(()); },
            incoming = listener.accept() => {
                let (stream, peer) = incoming?;
                let Ok(permit) = slots.clone().try_acquire_owned() else { continue; };
                let config = config.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(error) = protocol::handle_connection(stream, &config).await {
                        eprintln!("Connection {peer} ended: {error}");
                    }
                });
            }
        }
    }
}

async fn metrics_loop(
    interval_ms: u64,
    max_connections: usize,
    slots: Arc<Semaphore>,
    metrics: watch::Sender<Metrics>,
    mut stop: watch::Receiver<bool>,
    plugins: PluginManager,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut interval = time::interval(std::time::Duration::from_millis(50));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut window_start = Instant::now();
    let mut ticks_in_window = 0u64;
    let mut total_ticks = 0u64;
    let mut last_plugin_tick = 0u64;
    let mut system = sysinfo::System::new();
    let mut refresh = time::interval(std::time::Duration::from_millis(interval_ms.max(250)));
    loop {
        tokio::select! {
            _ = stop.changed() => if *stop.borrow() { return Ok(()); },
            _ = interval.tick() => {
                ticks_in_window += 1;
                total_ticks += 1;
                if total_ticks.saturating_sub(last_plugin_tick) >= 20 {
                    last_plugin_tick = total_ticks;
                    plugins.fire_tick(total_ticks);
                }
            }
            _ = refresh.tick() => {
                let elapsed = window_start.elapsed().as_secs_f64().max(0.001);
                system.refresh_all();
                let used_mb = system.processes().get(&sysinfo::get_current_pid()?)
                    .map_or(0, |process| process.memory() / (1024 * 1024));
                let _ = metrics.send(Metrics {
                    tps: (ticks_in_window as f64 / elapsed).min(20.0),
                    memory_mb: used_mb,
                    connections: max_connections - slots.available_permits(),
                    ticks: total_ticks,
                });
                ticks_in_window = 0;
                window_start = Instant::now();
            }
        }
    }
}

async fn console_loop(
    metrics: watch::Receiver<Metrics>,
    plugins: PluginManager,
    stop_tx: watch::Sender<bool>,
    mut stop: watch::Receiver<bool>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    loop {
        tokio::select! {
            _ = stop.changed() => if *stop.borrow() { return Ok(()); },
            line = lines.next_line() => match line? {
                Some(line) => {
                    let command = line.trim();
                    plugins.fire_command(command);
                    match command {
                        "status" | "tps" => {
                            let m = metrics.borrow();
                            let tps_bars = ((m.tps / 20.0) * 20.0).round().clamp(0.0, 20.0) as usize;
                            let ram_bars = (m.memory_mb / 128).min(20) as usize;
                            println!("TPS {:>4.1}/20 [{}{}] | RAM {} MiB [{}{}] | Players 0/{} | Connections {} | Tick {}",
                                m.tps, "█".repeat(tps_bars), "░".repeat(20 - tps_bars),
                                m.memory_mb, "█".repeat(ram_bars), "░".repeat(20 - ram_bars),
                                0, m.connections, m.ticks);
                        }
                        "help" => println!("Commands: status, tps, help, stop"),
                        "stop" | "shutdown" => { let _ = stop_tx.send(true); return Ok(()); }
                        "" => {}
                        other => println!("Unknown command '{other}'. Type help."),
                    }
                }
                None => return Ok(()),
            }
        }
    }
}
