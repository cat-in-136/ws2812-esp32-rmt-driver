//! Low-level LED pixel driver API.

pub mod color;

#[cfg_attr(not(target_vendor = "espressif"), path = "mock.rs")]
mod esp32_rmt;

pub use esp32_rmt::Ws2812Esp32RmtDriver;
pub use esp32_rmt::Ws2812Esp32RmtDriverError;
