use crate::lithummc;
use lithummc_wasm_host_common::{logging::log_tracing, state::PluginHostState};

impl lithummc::plugin::logging::Host for PluginHostState {
    fn log(
        &mut self,
        level: lithummc::plugin::logging::Level,
        message: String,
    ) -> wasmtime::Result<()> {
        match level {
            lithummc::plugin::logging::Level::Trace => tracing::trace!("[plugin] {message}"),
            lithummc::plugin::logging::Level::Debug => tracing::debug!("[plugin] {message}"),
            lithummc::plugin::logging::Level::Info => tracing::info!("[plugin] {message}"),
            lithummc::plugin::logging::Level::Warn => tracing::warn!("[plugin] {message}"),
            lithummc::plugin::logging::Level::Error => tracing::error!("[plugin] {message}"),
        }
        Ok(())
    }

    fn log_tracing(&mut self, event: Vec<u8>) -> wasmtime::Result<()> {
        log_tracing(&event);
        Ok(())
    }
}
