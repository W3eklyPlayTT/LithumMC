use crate::ClientPacket;
use lithummc_data::packet::clientbound::play::CLEAR_DIALOG;
use lithummc_macros::java_packet;
use lithummc_util::version::JavaMinecraftVersion;
#[java_packet(CLEAR_DIALOG)]
pub struct CPlayClearDialog;

impl CPlayClearDialog {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for CPlayClearDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientPacket for CPlayClearDialog {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
