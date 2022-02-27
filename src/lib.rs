#[cfg(feature = "unstable")]
pub mod driver;
#[cfg(not(feature = "unstable"))]
mod driver;

#[cfg(all(feature = "embedded-graphics-core", feature = "unstable"))]
pub mod lib_embedded_graphics;

#[cfg(all(feature = "smart-leds-trait", feature = "unstable"))]
pub mod lib_smart_leds;
#[cfg(all(feature = "smart-leds-trait", not(feature = "unstable")))]
mod lib_smart_leds;

#[cfg(feature = "smart-leds-trait")]
pub use lib_smart_leds::Ws2812Esp32Rmt;
#[cfg(feature = "smart-leds-trait")]
pub use smart_leds_trait::RGB8;
