//! LithumMC's public plugin API.
//!
//! This initial release re-exports the MIT OR Apache-2.0 WebAssembly
//! plugin API so plugins can use LithumMC package names while the server and API
//! mature. The underlying runtime ABI follows the LithumMC plugin API version
//! bundled by the server.

pub use lithummc_plugin_api_impl::*;
