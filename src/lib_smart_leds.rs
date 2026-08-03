//! smart-leds driver wrapper API.

#![allow(deprecated)] // Legacy RMT API is intentionally used when rmt-legacy feature is enabled

use crate::driver::color::{LedPixelColor, LedPixelColorGrb24, LedPixelColorImpl};
use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::marker::PhantomData;
use smart_leds_trait::RGB8;
#[cfg(feature = "alloc")]
use smart_leds_trait::SmartLedsWrite;
use smart_leds_trait::{RGBW};

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_hal;

// Import the appropriate types based on feature
#[cfg(not(feature = "rmt-legacy"))]
use esp_idf_hal::gpio::OutputPin;
#[cfg(feature = "rmt-legacy")]
use esp_idf_hal::gpio::OutputPin;
#[cfg(feature = "rmt-legacy")]
use esp_idf_hal::rmt::{RmtChannel, TxRmtDriver};

/// 8-bit RGBW (RGB + white)
pub type RGBW8 = RGBW<u8, u8>;

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > From<RGB8> for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    fn from(x: RGB8) -> Self {
        Self::new_with_rgb(x.r, x.g, x.b)
    }
}

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > From<RGBW8> for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    fn from(x: RGBW8) -> Self {
        Self::new_with_rgbw(x.r, x.g, x.b, x.a.0)
    }
}

/// ws2812-like smart led driver wrapper providing smart-leds API
///
/// This is a generalization to handle variants such as SK6812-RGBW 4-color LED.
/// Use [`Ws2812Esp32Rmt`] for typical RGB LED (WS2812B/SK6812) consisting of 8-bit GRB (total 24-bit pixel).
pub struct LedPixelEsp32Rmt<'d, CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    driver: Ws2812Esp32RmtDriver<'d>,
    phantom: PhantomData<(CSmart, CDev)>,
}

impl<'d, CSmart, CDev> LedPixelEsp32Rmt<'d, CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    /// Create a new driver wrapper (new API - no channel argument).
    #[cfg(not(feature = "rmt-legacy"))]
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Self::new_with_ws2812_driver(Ws2812Esp32RmtDriver::<'d>::new(pin)?)
    }

    /// Create a new driver wrapper (legacy API - takes channel).
    #[cfg(feature = "rmt-legacy")]
    pub fn new<C: RmtChannel + 'd>(
        channel: C,
        pin: impl OutputPin + 'd,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Self::new_with_ws2812_driver(Ws2812Esp32RmtDriver::<'d>::new(channel, pin)?)
    }

    /// Create a new driver wrapper with `TxRmtDriver` (legacy only).
    #[cfg(feature = "rmt-legacy")]
    pub fn new_with_rmt_driver(tx: TxRmtDriver<'d>) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Self::new_with_ws2812_driver(Ws2812Esp32RmtDriver::<'d>::new_with_rmt_driver(tx)?)
    }

    /// Create a new driver wrapper with `Ws2812Esp32RmtDriver`.
    pub fn new_with_ws2812_driver(
        driver: Ws2812Esp32RmtDriver<'d>,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ok(Self {
            driver,
            phantom: Default::default(),
        })
    }
}

impl<
        'd,
        CSmart,
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > LedPixelEsp32Rmt<'d, CSmart, LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>>
where
    LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>: From<CSmart>,
{
    /// Writes pixel data from a color sequence to the driver without data copy
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    pub fn write_nocopy<T, I>(&mut self, iterator: T) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        T: IntoIterator<Item = I>,
        I: Into<CSmart>,
        <T as IntoIterator>::IntoIter: Send,
    {
        self.driver
            .write_blocking(iterator.into_iter().flat_map(|color| {
                let c =
                    LedPixelColorImpl::<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>::from(color.into());
                c.0
            }))?;
        Ok(())
    }
}

// SmartLedsWrite impl for legacy mode (requires alloc)
#[cfg(all(feature = "rmt-legacy", feature = "alloc"))]
impl<'d, CSmart, CDev> SmartLedsWrite for LedPixelEsp32Rmt<'d, CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    type Error = Ws2812Esp32RmtDriverError;
    type Color = CSmart;

    /// Writes pixel data from a color sequence to the driver
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        let pixel_data = iterator.into_iter().fold(Vec::new(), |mut vec, color| {
            vec.extend_from_slice(CDev::from(color.into()).as_ref());
            vec
        });
        self.driver.write_blocking(pixel_data.into_iter())?;
        Ok(())
    }
}

// SmartLedsWrite impl for new_api mode (alloc always available)
#[cfg(not(feature = "rmt-legacy"))]
impl<'d, CSmart, CDev> SmartLedsWrite for LedPixelEsp32Rmt<'d, CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    type Error = Ws2812Esp32RmtDriverError;
    type Color = CSmart;

    /// Writes pixel data from a color sequence to the driver
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        let pixel_data = iterator.into_iter().fold(Vec::new(), |mut vec, color| {
            vec.extend_from_slice(CDev::from(color.into()).as_ref());
            vec
        });
        self.driver.write_blocking(pixel_data.into_iter())?;
        Ok(())
    }
}

/// 8-bit GRB (total 24-bit pixel) LED driver wrapper providing smart-leds API,
/// Typical RGB LED (WS2812B/SK6812) driver wrapper providing smart-leds API
pub type Ws2812Esp32Rmt<'d> = LedPixelEsp32Rmt<'d, RGB8, LedPixelColorGrb24>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::esp_idf_hal::peripherals::Peripherals;

    #[test]
    fn test_ws2812_esp32_rmt_smart_leds() {
        let sample_data = [RGB8::new(0x00, 0x01, 0x02), RGB8::new(0x03, 0x04, 0x05)];
        let expected_values: [u8; 6] = [0x01, 0x00, 0x02, 0x04, 0x03, 0x05];

        let peripherals = Peripherals::take().unwrap();
        let led_pin = peripherals.pins.gpio0;

        #[cfg(not(feature = "rmt-legacy"))]
        let mut ws2812 = Ws2812Esp32Rmt::new(led_pin).unwrap();
        #[cfg(feature = "rmt-legacy")]
        let mut ws2812 = Ws2812Esp32Rmt::new(peripherals.rmt.channel0, led_pin).unwrap();

        ws2812.write(sample_data.iter().cloned()).unwrap();
        assert_eq!(ws2812.driver.pixel_data.unwrap(), &expected_values);
    }
}
