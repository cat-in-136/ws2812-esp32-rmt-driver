//! smart-leds driver wrapper API.

use crate::driver::color::{LedPixelColor, LedPixelColorGrb24, LedPixelColorImpl};
use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::marker::PhantomData;
use esp_idf_hal::rmt::TxRmtDriver;
#[cfg(feature = "alloc")]
use smart_leds_trait::SmartLedsWrite;
use smart_leds_trait::{RGB8, RGBW};

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_hal;
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
///
/// This is a generalization to handle variants such as SK6812-RGBW 4-color LED.
/// Use [`Ws2812Esp32Rmt`] for typical RGB LED (WS2812B/SK6812) consisting of 8-bit GRB (total 24-bit pixel).
///
/// # Examples
///
/// ```
/// # #[cfg(not(target_vendor = "espressif"))]
/// # use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
/// #
/// use esp_idf_hal::peripherals::Peripherals;
/// use smart_leds::{SmartLedsWrite, White};
/// use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGBW8};
/// use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrbw32;
///
/// let peripherals = Peripherals::take().unwrap();
/// let led_pin = peripherals.pins.gpio26;
/// let channel = peripherals.rmt.channel0;
/// let mut ws2812 = LedPixelEsp32Rmt::<RGBW8, LedPixelColorGrbw32>::new(channel, led_pin).unwrap();
///
/// let pixels = std::iter::repeat(RGBW8 {r: 0, g: 0, b: 0, a: White(30)}).take(25);
/// ws2812.write(pixels).unwrap();
/// ```
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

    /// Create a new driver wrapper with `TxRmtDriver`.
    ///
    /// The clock divider must be set to 1 for the `driver` configuration.
    ///
    /// ```
    /// # #[cfg(not(target_vendor = "espressif"))]
    /// # use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
    /// #
    /// # use esp_idf_hal::peripherals::Peripherals;
    /// # use esp_idf_hal::rmt::config::TransmitConfig;
    /// # use esp_idf_hal::rmt::TxRmtDriver;
    /// #
    /// # let peripherals = Peripherals::take().unwrap();
    /// # let led_pin = peripherals.pins.gpio27;
    /// # let channel = peripherals.rmt.channel0;
    /// #
    /// let driver_config = TransmitConfig::new()
    ///     .clock_divider(1); // Required parameter.
    /// let driver = TxRmtDriver::new(channel, led_pin, &driver_config).unwrap();
    /// ```
    pub fn new_with_rmt_driver(tx: TxRmtDriver<'d>) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::<'d>::new_with_rmt_driver(tx)?;
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

#[cfg(feature = "alloc")]
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
///
/// # Examples
///
/// ```
/// # #[cfg(not(target_vendor = "espressif"))]
/// # use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
/// #
/// use esp_idf_hal::peripherals::Peripherals;
/// use smart_leds::{RGB8, SmartLedsWrite};
/// use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;
///
/// let peripherals = Peripherals::take().unwrap();
/// let led_pin = peripherals.pins.gpio27;
/// let channel = peripherals.rmt.channel0;
/// let mut ws2812 = Ws2812Esp32Rmt::new(channel, led_pin).unwrap();
///
/// let pixels = std::iter::repeat(RGB8::new(30, 0, 0)).take(25);
/// ws2812.write(pixels).unwrap();
/// ```
///
/// The LED colors may flicker randomly when using Wi-Fi or Bluetooth with Ws2812 LEDs.
/// This issue can be resolved by:
///
/// - Separating Wi-Fi/Bluetooth processing from LED control onto different cores.
/// - Use multiple memory blocks (memory symbols) in the RMT driver.
///
/// To do the second option, prepare `TxRmtDriver` yourself and
/// initialize with [`Self::new_with_rmt_driver`] as shown below.
///
/// ```
/// # #[cfg(not(target_vendor = "espressif"))]
/// # use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
/// #
/// # use esp_idf_hal::peripherals::Peripherals;
/// # use esp_idf_hal::rmt::config::TransmitConfig;
/// # use esp_idf_hal::rmt::TxRmtDriver;
/// # use smart_leds::{RGB8, SmartLedsWrite};
/// # use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;
/// #
/// # let peripherals = Peripherals::take().unwrap();
/// # let led_pin = peripherals.pins.gpio27;
/// # let channel = peripherals.rmt.channel0;
/// #
/// let driver_config = TransmitConfig::new()
///     .clock_divider(1)  // Required parameter.
///     .mem_block_num(2); // Increase the number depending on your code.
/// let driver = TxRmtDriver::new(channel, led_pin, &driver_config).unwrap();
///
/// let mut ws2812 = Ws2812Esp32Rmt::new_with_rmt_driver(driver).unwrap();
/// #
/// # let pixels = std::iter::repeat(RGB8::new(30, 0, 0)).take(25);
/// # ws2812.write(pixels).unwrap();
/// ```
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
        let channel = peripherals.rmt.channel0;

        let mut ws2812 = Ws2812Esp32Rmt::new(channel, led_pin).unwrap();
        ws2812.write(sample_data.iter().cloned()).unwrap();
        assert_eq!(ws2812.driver.pixel_data.unwrap(), &expected_values);
    }
}
