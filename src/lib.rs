#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod driver;

pub use driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};

#[cfg(feature = "embedded-graphics-core")]
pub mod lib_embedded_graphics;

#[cfg(feature = "smart-leds-trait")]
pub mod lib_smart_leds;

#[cfg(feature = "smart-leds-trait")]
pub use lib_smart_leds::{LedPixelEsp32Rmt, Ws2812Esp32Rmt, RGBW8};
#[cfg(feature = "smart-leds-trait")]
pub use smart_leds_trait::RGB8;
