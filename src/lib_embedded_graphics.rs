//! embedded-graphics draw target API.

use crate::driver::color::{LedPixelColor, LedPixelColorGrb24, LedPixelColorImpl};
use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Point, Size};
use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics_core::Pixel;
use std::marker::PhantomData;

#[cfg(target_vendor = "espressif")]
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

impl<const W: usize, const H: usize> LedPixelShape for LedPixelMatrix<W, H> {
    fn size() -> Size {
        Size::new(W as u32, H as u32)
    }

    fn pixel_index(point: Point) -> Option<usize> {
        if (0..W as i32).contains(&point.x) && (0..H as i32).contains(&point.y) {
            Some((point.x + point.y * W as i32) as usize)
        } else {
            None
        }
    }
}

/// Target for embedded-graphics drawing operations of the LED pixels.
///
/// * `CDraw` - color type for embedded-graphics drawing operations
/// * `CDev` - the LED pixel color type (device dependant). It shall be convertible from `CDraw`.
/// * `S` - the LED pixel shape
///
/// `flush()` operation shall be required to write changes from a framebuffer to the display.
pub struct LedPixelDrawTarget<'d, CDraw, CDev, S>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
{
    driver: Ws2812Esp32RmtDriver<'d>,
    data: Vec<u8>,
    brightness: u8,
    changed: bool,
    _phantom: PhantomData<(CDraw, CDev, S)>,
}

impl<'d, CDraw, CDev, S> LedPixelDrawTarget<'d, CDraw, CDev, S>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
{
    /// Create a new draw target.
    ///
    /// `channel` shall be different between different `pin`.
    #[cfg(target_vendor = "espressif")]
    pub fn new<C: RmtChannel>(
        channel: impl Peripheral<P = C> + 'd,
        pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::<'d>::new(channel, pin)?;
        let data = std::iter::repeat(0)
            .take(S::pixel_len() * CDev::BPP)
            .collect::<Vec<_>>();
        Ok(Self {
            driver,
            data,
            brightness: u8::MAX,
            changed: true,
            _phantom: Default::default(),
        })
    }

    /// Create a new draw target with dummy driver.
    #[cfg(not(target_vendor = "espressif"))]
    pub fn new() -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::<'d>::new()?;
        let data = std::iter::repeat(0)
            .take(S::pixel_len() * CDev::BPP)
            .collect::<Vec<_>>();
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

impl<'d, CDraw, CDev, S> OriginDimensions for LedPixelDrawTarget<'d, CDraw, CDev, S>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
{
    #[inline]
    fn size(&self) -> Size {
        S::size()
    }
}

impl<'d, CDraw, CDev, S> DrawTarget for LedPixelDrawTarget<'d, CDraw, CDev, S>
where
    CDraw: RgbColor,
    CDev: LedPixelColor + From<CDraw>,
    S: LedPixelShape,
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
/// 24bit GRB LED (Typical RGB LED (WS2812BsSK6812)) draw target
pub type Ws2812DrawTarget<'d, S> = LedPixelDrawTarget<'d, Rgb888, LedPixelColorGrb24, S>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_led_pixel_matrix() {
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
        let draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>>::new().unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(
            draw.data,
            std::iter::repeat(0).take(150).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ws2812draw_target_draw() {
        let mut draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>>::new().unwrap();

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
            std::iter::repeat([0x08, 0x07, 0x0A])
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
        let mut draw = Ws2812DrawTarget::<LedPixelMatrix<10, 5>>::new().unwrap();

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
