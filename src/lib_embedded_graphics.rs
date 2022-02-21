use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Point, Size};
use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics_core::Pixel;
use std::marker::PhantomData;

pub trait ColorValueConverter<C: RgbColor> {
    const BPP: usize;
    fn convert(color: C, i: usize) -> Option<u8>;
}

struct Ws2812Grb24ColorValueConverter<C: RgbColor> {
    _phantom_color: PhantomData<C>,
}

impl<C: RgbColor> ColorValueConverter<C> for Ws2812Grb24ColorValueConverter<C> {
    const BPP: usize = 3;
    fn convert(color: C, i: usize) -> Option<u8> {
        match i {
            0 => Some(color.g()),
            1 => Some(color.r()),
            2 => Some(color.b()),
            _ => None,
        }
    }
}

pub trait LedPixelShape<C: RgbColor, CC: ColorValueConverter<C>> {
    fn data_len() -> usize {
        let size = Self::size();
        (size.width * size.height) as usize * CC::BPP
    }
    fn size() -> Size;
    fn convert(point: Point) -> Option<usize>;
}

pub struct LedPixelMatrixShape<
    C: RgbColor,
    CC: ColorValueConverter<C>,
    const W: usize,
    const H: usize,
> {
    _phantom_color: PhantomData<C>,
    _phantom_converter: PhantomData<CC>,
}

impl<C: RgbColor, CC: ColorValueConverter<C>, const W: usize, const H: usize> LedPixelShape<C, CC>
    for LedPixelMatrixShape<C, CC, W, H>
{
    fn size() -> Size {
        Size::new(W as u32, H as u32)
    }

    fn convert(point: Point) -> Option<usize> {
        if (0..W as i32).contains(&point.x) && (0..H as i32).contains(&point.y) {
            Some(CC::BPP * (point.x + point.y * W as i32) as usize)
        } else {
            None
        }
    }
}

pub struct LedPixelDrawTarget<C: RgbColor, CC: ColorValueConverter<C>, S: LedPixelShape<C, CC>> {
    driver: Ws2812Esp32RmtDriver,
    data: Vec<u8>,
    changed: bool,
    _phantom_color: PhantomData<C>,
    _phantom_converter: PhantomData<CC>,
    _phantom_shape: PhantomData<S>,
}

impl<C: RgbColor, CC: ColorValueConverter<C>, S: LedPixelShape<C, CC>>
    LedPixelDrawTarget<C, CC, S>
{
    pub fn new(channel_num: u8, gpio_num: u32) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::new(channel_num, gpio_num)?;
        let data = std::iter::repeat(0).take(S::data_len()).collect::<Vec<_>>();
        Ok(Self {
            driver,
            data,
            changed: true,
            _phantom_color: Default::default(),
            _phantom_converter: Default::default(),
            _phantom_shape: Default::default(),
        })
    }

    pub fn clear_with_black(&mut self) -> Result<(), Ws2812Esp32RmtDriverError> {
        self.data.fill(0);
        self.changed = true;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Ws2812Esp32RmtDriverError> {
        if self.changed {
            self.driver.write(&self.data)?;
            self.changed = false;
        }
        Ok(())
    }
}

impl<C: RgbColor, CC: ColorValueConverter<C>, S: LedPixelShape<C, CC>> OriginDimensions
    for LedPixelDrawTarget<C, CC, S>
{
    fn size(&self) -> Size {
        S::size()
    }
}

impl<C: RgbColor, CC: ColorValueConverter<C>, S: LedPixelShape<C, CC>> DrawTarget
    for LedPixelDrawTarget<C, CC, S>
{
    type Color = C;
    type Error = Ws2812Esp32RmtDriverError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if let Some(index) = S::convert(point) {
                for offset in 0..CC::BPP {
                    if let Some(v) = CC::convert(color, offset) {
                        self.data[index + offset] = v;
                    }
                }
                self.changed = true;
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let c = (0..CC::BPP)
            .map(|i| CC::convert(color, i).unwrap())
            .collect::<Vec<_>>();
        for (index, v) in self.data.iter_mut().enumerate() {
            *v = c[index % CC::BPP];
        }
        self.changed = true;
        Ok(())
    }
}

type Ws2812MatrixShape<const W: usize, const H: usize> =
    LedPixelMatrixShape<Rgb888, Ws2812Grb24ColorValueConverter<Rgb888>, W, H>;
type Ws2812StripShape<const L: usize> = Ws2812MatrixShape<L, 1>;
type Ws2812DrawTarget<S> = LedPixelDrawTarget<Rgb888, Ws2812Grb24ColorValueConverter<Rgb888>, S>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ws2812matrix_shape() {
        assert_eq!(Ws2812MatrixShape::<10, 5>::data_len(), 150);
        assert_eq!(Ws2812MatrixShape::<10, 5>::size(), Size::new(10, 5));
        assert_eq!(
            Ws2812MatrixShape::<10, 5>::convert(Point::new(0, 0)),
            Some(0)
        );
        assert_eq!(
            Ws2812MatrixShape::<10, 5>::convert(Point::new(9, 4)),
            Some(147)
        );
        assert_eq!(Ws2812MatrixShape::<10, 5>::convert(Point::new(-1, 0)), None);
        assert_eq!(Ws2812MatrixShape::<10, 5>::convert(Point::new(0, -1)), None);
        assert_eq!(Ws2812MatrixShape::<10, 5>::convert(Point::new(10, 4)), None);
        assert_eq!(Ws2812MatrixShape::<10, 5>::convert(Point::new(9, 5)), None);
    }

    #[test]
    fn test_ws2812strip_shape() {
        assert_eq!(Ws2812StripShape::<10>::data_len(), 30);
        assert_eq!(Ws2812StripShape::<10>::size(), Size::new(10, 1));
        assert_eq!(Ws2812StripShape::<10>::convert(Point::new(0, 0)), Some(0));
        assert_eq!(Ws2812StripShape::<10>::convert(Point::new(9, 0)), Some(27));
        assert_eq!(Ws2812StripShape::<10>::convert(Point::new(-1, 0)), None);
        assert_eq!(Ws2812StripShape::<10>::convert(Point::new(0, -1)), None);
        assert_eq!(Ws2812StripShape::<10>::convert(Point::new(10, 0)), None);
        assert_eq!(Ws2812StripShape::<10>::convert(Point::new(9, 1)), None);
    }

    #[test]
    fn test_ws2812draw_target_new() {
        let draw = Ws2812DrawTarget::<Ws2812MatrixShape<10, 5>>::new(0, 27).unwrap();
        assert_eq!(draw.changed, true);
        assert_eq!(
            draw.data,
            std::iter::repeat(0).take(150).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ws2812draw_target_draw() {
        let mut draw = Ws2812DrawTarget::<Ws2812MatrixShape<10, 5>>::new(0, 27).unwrap();

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
        let mut draw = Ws2812DrawTarget::<Ws2812MatrixShape<10, 5>>::new(0, 27).unwrap();

        draw.changed = true;
        draw.data.fill(0x01);
        draw.driver.grb_pixels = None;
        draw.flush().unwrap();
        assert_eq!(draw.driver.grb_pixels.unwrap(), draw.data);
        assert_eq!(draw.changed, false);

        draw.driver.grb_pixels = None;
        draw.flush().unwrap();
        assert_eq!(draw.driver.grb_pixels, None);
        assert_eq!(draw.changed, false);
    }
}