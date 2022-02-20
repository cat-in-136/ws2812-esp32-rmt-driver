#[cfg(target_vendor = "espressif")]
mod esp32_rmt;
#[cfg(not(target_vendor = "espressif"))]
mod mock;

#[cfg(not(target_vendor = "espressif"))]
use mock as esp32_rmt;

pub use esp32_rmt::Ws2812Esp32RmtDriver;
pub use esp32_rmt::Ws2812Esp32RmtDriverError;
