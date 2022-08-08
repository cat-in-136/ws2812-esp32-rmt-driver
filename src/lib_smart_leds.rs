//! smart-leds driver wrapper API.

use crate::driver::color::{LedPixelColor, LedPixelColorGrb24, LedPixelColorImpl};
use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};
use std::marker::PhantomData;

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
pub struct LedPixelEsp32Rmt<CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    driver: Ws2812Esp32RmtDriver,
    phantom: PhantomData<(CSmart, CDev)>,
}

impl<CSmart, CDev> LedPixelEsp32Rmt<CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    /// Create a new driver wrapper.
    ///
    /// `channel_num` shall be different between different `gpio_num`.
    pub fn new(channel_num: u8, gpio_num: u32) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::new(channel_num, gpio_num)?;
        Ok(Self {
            driver,
            phantom: Default::default(),
        })
    }
}

impl<CSmart, CDev> SmartLedsWrite for LedPixelEsp32Rmt<CSmart, CDev>
where
    CDev: LedPixelColor + From<CSmart>,
{
    type Error = Ws2812Esp32RmtDriverError;
    type Color = CSmart;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        let mut pixel_data = Vec::new();
        for color in iterator {
            for v in CDev::from(color.into()).as_ref() {
                pixel_data.push(*v)
            }
        }
        self.driver.write(pixel_data.as_slice())
    }
}

/// ws2812 driver wrapper providing smart-leds API
pub type Ws2812Esp32Rmt = LedPixelEsp32Rmt<RGB8, LedPixelColorGrb24>;

#[test]
#[cfg(not(target_vendor = "espressif"))]
fn test_ws2812_esp32_rmt_smart_leds() {
    let sample_data = [RGB8::new(0x00, 0x01, 0x02), RGB8::new(0x03, 0x04, 0x05)];
    let expected_values: [u8; 6] = [0x01, 0x00, 0x02, 0x04, 0x03, 0x05];
    let mut ws2812 = Ws2812Esp32Rmt::new(0, 27).unwrap();
    ws2812.write(sample_data.iter().cloned()).unwrap();
    assert_eq!(ws2812.driver.pixel_data.unwrap(), &expected_values);
}
