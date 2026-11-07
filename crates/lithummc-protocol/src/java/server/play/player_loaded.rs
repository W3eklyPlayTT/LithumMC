use crate::{ServerPacket, ser::ReadingError};
use lithummc_data::packet::serverbound::play::PLAYER_LOADED;
use lithummc_macros::java_packet;
use lithummc_util::version::JavaMinecraftVersion;

/// Added in 1.21.4
#[java_packet(PLAYER_LOADED)]
pub struct SPlayerLoaded;

impl<'a> ServerPacket<'a> for SPlayerLoaded {
    fn read(
        _bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SPlayerLoaded {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
