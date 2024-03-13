//! smart-leds driver wrapper API.

use crate::driver::color::{LedPixelColor, LedPixelColorGrb24, LedPixelColorImpl};
use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};
use std::marker::PhantomData;

#[cfg(target_vendor = "espressif")]
use esp_idf_hal::{gpio::OutputPin, peripheral::Peripheral, rmt::RmtChannel};

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
    /// Create a new driver wrapper.
    ///
    /// `channel` shall be different between different `pin`.
    #[cfg(target_vendor = "espressif")]
    pub fn new<C: RmtChannel>(
        channel: impl Peripheral<P = C> + 'd,
        pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::<'d>::new(channel, pin)?;
        Ok(Self {
            driver,
            phantom: Default::default(),
        })
    }

    /// Create a new driver wrapper with dummy driver.
    #[cfg(not(target_vendor = "espressif"))]
    pub fn new() -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::<'d>::new()?;
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

/// ws2812 driver wrapper providing smart-leds API
pub type Ws2812Esp32Rmt<'d> = LedPixelEsp32Rmt<'d, RGB8, LedPixelColorGrb24>;

#[test]
#[cfg(not(target_vendor = "espressif"))]
fn test_ws2812_esp32_rmt_smart_leds() {
    let sample_data = [RGB8::new(0x00, 0x01, 0x02), RGB8::new(0x03, 0x04, 0x05)];
    let expected_values: [u8; 6] = [0x01, 0x00, 0x02, 0x04, 0x03, 0x05];
    let mut ws2812 = Ws2812Esp32Rmt::new().unwrap();
    ws2812.write(sample_data.iter().cloned()).unwrap();
    assert_eq!(ws2812.driver.pixel_data.unwrap(), &expected_values);
}
