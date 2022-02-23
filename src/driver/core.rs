pub trait LedPixelColor:
    Ord + PartialOrd + Eq + PartialEq + Clone + AsRef<[u8]> + AsMut<[u8]>
{
    const BPP: usize;
    fn new_with_rgb(r: u8, g: u8, b: u8) -> Self;
    fn new_with_rgbw(r: u8, g: u8, b: u8, _w: u8) -> Self {
        Self::new_with_rgb(r, g, b)
    }
}

mod ws2812grb24 {
    use super::*;

    #[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Default)]
    #[repr(transparent)]
    pub struct Ws2812Grb24Color([u8; 3]);

    impl LedPixelColor for Ws2812Grb24Color {
        const BPP: usize = 3;
        #[inline]
        fn new_with_rgb(r: u8, g: u8, b: u8) -> Self {
            Self([g, r, b])
        }
    }

    impl AsRef<[u8]> for Ws2812Grb24Color {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    impl AsMut<[u8]> for Ws2812Grb24Color {
        #[inline]
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }
    }

    #[test]
    fn test_ws2812grb24color() {
        let color = Ws2812Grb24Color::new_with_rgb(1, 2, 3);
        assert_eq!(color.0, [2, 1, 3]);
        assert_eq!(color.as_ref(), &color.0);

        let color = Ws2812Grb24Color::new_with_rgbw(1, 2, 3, 4);
        assert_eq!(color.0, [2, 1, 3]);
        assert_eq!(color.as_ref(), &color.0);
    }
}

mod sk6812rgbw32 {
    use super::*;

    #[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Default)]
    #[repr(transparent)]
    pub struct Sk6812Rgbw32Color([u8; 4]);

    impl LedPixelColor for Sk6812Rgbw32Color {
        const BPP: usize = 4;
        #[inline]
        fn new_with_rgb(r: u8, g: u8, b: u8) -> Self {
            Self::new_with_rgbw(r, g, b, 0)
        }

        #[inline]
        fn new_with_rgbw(r: u8, g: u8, b: u8, w: u8) -> Self {
            Self([r, g, b, w])
        }
    }

    impl AsRef<[u8]> for Sk6812Rgbw32Color {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    impl AsMut<[u8]> for Sk6812Rgbw32Color {
        #[inline]
        fn as_mut(&mut self) -> &mut [u8] {
            &mut self.0
        }
    }

    #[test]
    fn test_sk6812rgb32color() {
        let color = Sk6812Rgbw32Color::new_with_rgb(1, 2, 3);
        assert_eq!(color.0, [1, 2, 3, 0]);
        assert_eq!(color.as_ref(), &color.0);

        let color = Sk6812Rgbw32Color::new_with_rgbw(1, 2, 3, 4);
        assert_eq!(color.0, [1, 2, 3, 4]);
        assert_eq!(color.as_ref(), &color.0);
    }
}

pub use sk6812rgbw32::Sk6812Rgbw32Color;
pub use ws2812grb24::Ws2812Grb24Color;
