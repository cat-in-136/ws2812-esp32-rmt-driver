//! Low-level LED pixel driver API.

pub mod color;

#[cfg(feature = "rmt-legacy")]
mod legacy;

#[cfg(feature = "rmt-legacy")]
use legacy::esp32_rmt;

// Ensure alloc is enabled when using new API
#[cfg(all(not(feature = "rmt-legacy"), not(feature = "alloc")))]
compile_error!("The new RMT API requires the alloc feature");

#[cfg(not(feature = "rmt-legacy"))]
mod new_api;

#[cfg(not(feature = "rmt-legacy"))]
use new_api::esp32_rmt;

pub use esp32_rmt::Ws2812Esp32RmtDriver;
pub use esp32_rmt::Ws2812Esp32RmtDriverBuilder;
pub use esp32_rmt::Ws2812Esp32RmtDriverError;

#[cfg(not(feature = "rmt-legacy"))]
pub use esp32_rmt::Ws2812Esp32RmtTxQueue;
