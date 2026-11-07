use lithummc_data::packet::clientbound::play::BUNDLE_DELIMITER;
use lithummc_macros::java_packet;

use crate::ClientPacket;
use lithummc_util::version::JavaMinecraftVersion;

#[java_packet(BUNDLE_DELIMITER)]
pub struct CBundleDelimiter;

impl ClientPacket for CBundleDelimiter {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
