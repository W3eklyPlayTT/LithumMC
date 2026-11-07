//! Lithum's public plugin API.
//!
//! This initial release re-exports Pumpkin's MIT OR Apache-2.0 WebAssembly
//! plugin API so plugins can use Lithum package names while the server and API
//! mature. The underlying runtime ABI follows the Pumpkin plugin API version
//! bundled by the server.

pub use pumpkin_plugin_api::*;
