//! Low-level LED pixel driver API.

pub mod color;
mod esp32_rmt;

pub use esp32_rmt::Ws2812Esp32RmtDriver;
pub use esp32_rmt::Ws2812Esp32RmtDriverError;
