use lithummc_data::packet::clientbound::config::RESET_CHAT;
use lithummc_macros::java_packet;

use crate::ClientPacket;
use lithummc_util::version::JavaMinecraftVersion;

#[java_packet(RESET_CHAT)]
pub struct CConfigResetChat;

impl ClientPacket for CConfigResetChat {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
