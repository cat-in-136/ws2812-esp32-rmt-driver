//! embedded-graphics draw target API.

use crate::driver::color::{LedPixelColor, LedPixelColorGrb24, LedPixelColorImpl};
use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
use core::marker::PhantomData;
use core::ops::DerefMut;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Point, Size};
use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics_core::Pixel;

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_hal;
use esp_idf_hal::{gpio::OutputPin, peripheral::Peripheral, rmt::RmtChannel};

/// LED pixel shape
pub trait LedPixelShape {
    /// Returns the number of pixels
    fn pixel_len() -> usize {
        let size = Self::size();
        (size.width * size.height) as usize
    }
    /// Physical size of the LED pixel equipment.
    fn size() -> Size;
    /// Convert from `point` to the index.
    /// Returns `None` if it is out of the bounds.
    fn pixel_index(point: Point) -> Option<usize>;
}

/// LED pixel shape of `W`x`H` matrix
pub struct LedPixelMatrix<const W: usize, const H: usize> {}

impl<const W: usize, const H: usize> LedPixelMatrix<W, H> {
    /// Physical size of the LED pixel matrix.
    pub const SIZE: Size = Size::new(W as u32, H as u32);
    /// The number of pixels.
    pub const PIXEL_LEN: usize = W * H;
}

impl<const W: usize, const H: usize> LedPixelShape for LedPixelMatrix<W, H> {
    #[inline]
    fn size() -> Size {
        Self::SIZE
    }
    #[inline]
    fn pixel_len() -> usize {
        Self::PIXEL_LEN
    }

    fn pixel_index(point: Point) -> Option<usize> {
        if (0..W as i32).contains(&point.x) && (0..H as i32).contains(&point.y) {
            Some((point.x + point.y * W as i32) as usize)
        } else {
            None
        }
    }
}

/// Default data storage type for `LedPixelDrawTarget`.
#[cfg(feature = "std")]
type LedPixelDrawTargetData = Vec<u8>;

/// Default data storage type for `LedPixelDrawTarget`.
#[cfg(all(not(feature = "std"), feature = "alloc"))]
type LedPixelDrawTargetData = alloc::vec::Vec<u8>;

/// Default data storage type for `LedPixelDrawTarget`.
/// In case of heapless, allocate 256-byte capacity vector.
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type LedPixelDrawTargetData = heapless::Vec<u8, 256>;

/// Target for embedded-graphics drawing operations of the LED pixels.
///
/// This is a generalization for the future extension.
/// Use [`Ws2812DrawTarget`] for typical RGB LED (WS2812B/SK6812) consisting of 8-bit GRB (total 24-bit pixel).
///
/// * `CDraw` - color type for embedded-graphics drawing operations
/// * `CDev` - the LED pixel color type (device dependant). It shall be convertible from `CDraw`.
/// * `S` - the LED pixel shape
/// * `Data` - (optional) data storage type. It shall be `Vec`-like struct.
///
/// [`flush()`] operation shall be required to write changes from a framebuffer to the display.
///
/// For non-`alloc` no_std environment, `Data` should be explicitly set to some `Vec`-like struct:
/// e.g., `heapless::Vec<u8, PIXEL_LEN>` where `PIXEL_LEN` equals to `S::size() * CDev::BPP`.
///
/// [`flush()`]: #method.flush
pub struct LedPixelDrawTarget<'d, CDraw, CDev, S, Data = LedPixelDrawTargetData>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
    Data: DerefMut<Target = [u8]> + FromIterator<u8> + IntoIterator<Item = u8>,
{
    driver: Ws2812Esp32RmtDriver<'d>,
    data: Data,
    brightness: u8,
    changed: bool,
    _phantom: PhantomData<(CDraw, CDev, S, Data)>,
}

impl<'d, CDraw, CDev, S, Data> LedPixelDrawTarget<'d, CDraw, CDev, S, Data>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
    Data: DerefMut<Target = [u8]> + FromIterator<u8> + IntoIterator<Item = u8>,
{
    /// Create a new draw target.
    ///
    /// `channel` shall be different between different `pin`.
    pub fn new<C: RmtChannel>(
        channel: impl Peripheral<P = C> + 'd,
        pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::<'d>::new(channel, pin)?;
        let data = core::iter::repeat(0)
            .take(S::pixel_len() * CDev::BPP)
            .collect::<Data>();
        Ok(Self {
            driver,
            data,
            brightness: u8::MAX,
            changed: true,
            _phantom: Default::default(),
        })
    }

    /// Set maximum brightness.
    /// Each channel values of the returned shall be scaled down to `(brightness + 1) / 256`.
    #[inline]
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
        self.changed = true;
    }

    /// Returns maximum brightness.
    #[inline]
    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    /// Clear with black.
    /// Same operation as `clear(black_color)`.
    pub fn clear_with_black(&mut self) -> Result<(), Ws2812Esp32RmtDriverError> {
        self.data.fill(0);
        self.changed = true;
        Ok(())
    }

    /// Write changes from a framebuffer to the LED pixels
    pub fn flush(&mut self) -> Result<(), Ws2812Esp32RmtDriverError> {
        if self.changed {
            self.driver.write_blocking(self.data.iter().copied())?;
            self.changed = false;
        }
        Ok(())
    }
}

impl<'d, CDraw, CDev, S, Data> OriginDimensions for LedPixelDrawTarget<'d, CDraw, CDev, S, Data>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
    Data: DerefMut<Target = [u8]> + FromIterator<u8> + IntoIterator<Item = u8>,
{
    #[inline]
    fn size(&self) -> Size {
        S::size()
    }
}

impl<'d, CDraw, CDev, S, Data> DrawTarget for LedPixelDrawTarget<'d, CDraw, CDev, S, Data>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
    Data: DerefMut<Target = [u8]> + FromIterator<u8> + IntoIterator<Item = u8>,
{
    type Color = CDraw;
    type Error = Ws2812Esp32RmtDriverError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if let Some(pixel_index) = S::pixel_index(point) {
                let index = pixel_index * CDev::BPP;
                let color_device = CDev::from(color).brightness(self.brightness);
                for (offset, v) in color_device.as_ref().iter().enumerate() {
                    self.data[index + offset] = *v;
                }
                self.changed = true;
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let c = CDev::from(color).brightness(self.brightness);
        for (index, v) in self.data.iter_mut().enumerate() {
            *v = c.as_ref()[index % CDev::BPP];
        }
        self.changed = true;
        Ok(())
    }
}

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > From<Rgb888> for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    fn from(x: Rgb888) -> Self {
        Self::new_with_rgb(x.r(), x.g(), x.b())
    }
}

/// LED pixel shape of `L`-led strip
pub type LedPixelStrip<const L: usize> = LedPixelMatrix<L, 1>;

/// 8-bit GRB (total 24-bit pixel) LED draw target, Typical RGB LED (WS2812B/SK6812) draw target
///
/// * `S` - the LED pixel shape
/// * `Data` - (optional) data storage type. It shall be `Vec`-like struct.
///
/// [`flush()`] operation shall be required to write changes from a framebuffer to the display.
///
/// For non-`alloc` no_std environment, `Data` should be explicitly set to some `Vec`-like struct:
/// e.g., `heapless::Vec<u8, PIXEL_LEN>` where `PIXEL_LEN` equals to `S::size() * LedPixelColorGrb24::BPP`.
///
/// [`flush()`]: #method.flush
///
/// # Examples
///
/// ```
/// #[cfg(not(target_vendor = "espressif"))]
/// use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
///
/// use embedded_graphics::pixelcolor::Rgb888;
/// use embedded_graphics::prelude::*;
/// use embedded_graphics::primitives::{Circle, PrimitiveStyle};
/// use esp_idf_hal::peripherals::Peripherals;
/// use ws2812_esp32_rmt_driver::lib_embedded_graphics::{LedPixelMatrix, Ws2812DrawTarget};
///
/// let peripherals = Peripherals::take().unwrap();
/// let led_pin = peripherals.pins.gpio27;
/// let channel = peripherals.rmt.channel0;
/// let mut draw = Ws2812DrawTarget::<LedPixelMatrix<5, 5>>::new(channel, led_pin).unwrap();
/// draw.set_brightness(40);
/// draw.clear_with_black().unwrap();
/// let mut translated_draw = draw.translated(Point::new(0, 0));
/// Circle::new(Point::new(0, 0), 5)
///     .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
///     .draw(&mut translated_draw)
///     .unwrap();
/// draw.flush().unwrap();
/// ```
pub type Ws2812DrawTarget<'d, S, Data = LedPixelDrawTargetData> =
    LedPixelDrawTarget<'d, Rgb888, LedPixelColorGrb24, S, Data>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::esp_idf_hal::peripherals::Peripherals;

    #[test]
    fn test_led_pixel_matrix() {
        assert_eq!(LedPixelMatrix::<10, 5>::PIXEL_LEN, 50);
        assert_eq!(LedPixelMatrix::<10, 5>::SIZE, Size::new(10, 5));
        assert_eq!(LedPixelMatrix::<10, 5>::pixel_len(), 50);
        assert_eq!(LedPixelMatrix::<10, 5>::size(), Size::new(10, 5));
        assert_eq!(
            LedPixelMatrix::<10, 5>::pixel_index(Point::new(0, 0)),
            Some(0)
        );
        assert_eq!(
            LedPixelMatrix::<10, 5>::pixel_index(Point::new(9, 4)),
            Some(49)
        );
        assert_eq!(
            LedPixelMatrix::<10, 5>::pixel_index(Point::new(-1, 0)),
            None
        );
        assert_eq!(
            LedPixelMatrix::<10, 5>::pixel_index(Point::new(0, -1)),
            None
        );
        assert_eq!(
            LedPixelMatrix::<10, 5>::pixel_index(Point::new(10, 4)),
            None
        );
        assert_eq!(LedPixelMatrix::<10, 5>::pixel_index(Point::new(9, 5)), None);
    }

    #[test]
    fn test_led_pixel_strip() {
        assert_eq!(LedPixelStrip::<10>::PIXEL_LEN, 10);
        assert_eq!(LedPixelStrip::<10>::SIZE, Size::new(10, 1));
        assert_eq!(LedPixelStrip::<10>::pixel_len(), 10);
        assert_eq!(LedPixelStrip::<10>::size(), Size::new(10, 1));
        assert_eq!(LedPixelStrip::<10>::pixel_index(Point::new(0, 0)), Some(0));
        assert_eq!(LedPixelStrip::<10>::pixel_index(Point::new(9, 0)), Some(9));
        assert_eq!(LedPixelStrip::<10>::pixel_index(Point::new(-1, 0)), None);
        assert_eq!(LedPixelStrip::<10>::pixel_index(Point::new(0, -1)), None);
        assert_eq!(LedPixelStrip::<10>::pixel_index(Point::new(10, 0)), None);
        assert_eq!(LedPixelStrip::<10>::pixel_index(Point::new(9, 1)), None);
    }

    #[test]
    fn test_ws2812draw_target_new() {
        let peripherals = Peripherals::take().unwrap();
        let led_pin = peripherals.pins.gpio0;
        let channel = peripherals.rmt.channel0;

        let draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>>::new(channel, led_pin).unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(
            draw.data,
            core::iter::repeat(0).take(150).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ws2812draw_target_new_with_custom_data_struct() {
        const VEC_CAPACITY: usize = LedPixelMatrix::<10, 5>::PIXEL_LEN * LedPixelColorGrb24::BPP;

        let peripherals = Peripherals::take().unwrap();
        let led_pin = peripherals.pins.gpio0;
        let channel = peripherals.rmt.channel0;

        let draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>, heapless::Vec<u8, VEC_CAPACITY>>::new(
            channel, led_pin,
        )
        .unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(
            draw.data,
            core::iter::repeat(0)
                .take(150)
                .collect::<heapless::Vec<_, VEC_CAPACITY>>()
        );
    }

    #[test]
    fn test_ws2812draw_target_draw() {
        let peripherals = Peripherals::take().unwrap();
        let led_pin = peripherals.pins.gpio1;
        let channel = peripherals.rmt.channel1;

        let mut draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>>::new(channel, led_pin).unwrap();

        draw.draw_iter(
            [
                Pixel(Point::new(0, 0), Rgb888::new(0x01, 0x02, 0x03)),
                Pixel(Point::new(9, 4), Rgb888::new(0x04, 0x05, 0x06)),
                Pixel(Point::new(10, 5), Rgb888::new(0xFF, 0xFF, 0xFF)), // out of matrix shape
            ]
            .iter()
            .cloned(),
        )
        .unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(draw.data[0..3], [0x02, 0x01, 0x03]);
        assert_eq!(draw.data[3..147], [0x00; 144]);
        assert_eq!(draw.data[147..150], [0x05, 0x04, 0x06]);
        draw.changed = false;

        draw.clear(Rgb888::new(0x07, 0x08, 0x0A)).unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(
            draw.data,
            core::iter::repeat([0x08, 0x07, 0x0A])
                .take(50)
                .flatten()
                .collect::<Vec<_>>()
        );
        draw.changed = false;

        draw.clear_with_black().unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(draw.data, [0x00; 150]);
        draw.changed = false;
    }

    #[test]
    fn test_ws2812draw_target_flush() {
        let peripherals = Peripherals::take().unwrap();
        let led_pin = peripherals.pins.gpio2;
        let channel = peripherals.rmt.channel2;

        let mut draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>>::new(channel, led_pin).unwrap();

        draw.changed = true;
        draw.data.fill(0x01);
        draw.driver.pixel_data = None;
        draw.flush().unwrap();
        assert_eq!(draw.driver.pixel_data.unwrap(), draw.data);
        assert_eq!(draw.changed, false);

        draw.driver.pixel_data = None;
        draw.flush().unwrap();
        assert_eq!(draw.driver.pixel_data, None);
        assert_eq!(draw.changed, false);
    }
}
