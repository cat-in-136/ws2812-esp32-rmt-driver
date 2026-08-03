//! Low-level LED pixel driver API.

pub mod color;

#[cfg(all(not(feature = "rmt-legacy")))]
compile_error!("Not Implemented yet");

#[cfg(feature = "rmt-legacy")]
mod legacy;

#[cfg(feature = "rmt-legacy")]
use legacy::esp32_rmt as esp32_rmt;

pub use esp32_rmt::Ws2812Esp32RmtDriver;
pub use esp32_rmt::Ws2812Esp32RmtDriverBuilder;
pub use esp32_rmt::Ws2812Esp32RmtDriverError;
