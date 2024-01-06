//! device-dependant LED pixel colors

/// LED pixel color trait
pub trait LedPixelColor:
    Ord + PartialOrd + Eq + PartialEq + Clone + Sync + AsRef<[u8]> + AsMut<[u8]>
{
    /// byte per pixel. e.g. 3 for typical RGB.
    const BPP: usize;
    /// Creates with RGB (Red-Green-Blue) value.
    fn new_with_rgb(r: u8, g: u8, b: u8) -> Self;
    /// Creates with RGBW (Red-Green-Blue, and White) value.
    fn new_with_rgbw(r: u8, g: u8, b: u8, w: u8) -> Self;
    /// Returns Red channel value
    fn r(&self) -> u8;
    /// Returns Green channel value
    fn g(&self) -> u8;
    /// Returns Blue channel value
    fn b(&self) -> u8;
    /// Returns White channel value
    fn w(&self) -> u8;

    /// Returns brightness-adjusted color.
    /// Each channel values of the returned shall be scaled down to `(brightness + 1) / 256`.
    #[inline]
    fn brightness(&self, brightness: u8) -> Self {
        Self::new_with_rgbw(
            ((self.r() as u16) * (brightness as u16 + 1) / 256) as u8,
            ((self.g() as u16) * (brightness as u16 + 1) / 256) as u8,
            ((self.b() as u16) * (brightness as u16 + 1) / 256) as u8,
            ((self.w() as u16) * (brightness as u16 + 1) / 256) as u8,
        )
    }
}

/// LED pixel color struct made with an `N`-length `u8` array.
///
/// * `N` - Byte per pixel. equals to [`BPP`](#associatedconstant.BPP).
/// * `R_ORDER` - Index of the Red. Specify the value larger than `N - 1` if absent.
/// * `G_ORDER` - Index of the Green. Specify the value larger than `N - 1` if absent.
/// * `B_ORDER` - Index of the Blue. Specify the value larger than `N - 1` if absent.
/// * `W_ORDER` - Index of the White. Specify the value larger than `N - 1` if absent.
///
/// # Examples
///
/// ```
/// let color = LedPixelColorImpl::<3, 1, 0, 2, 255>::new_with_rgb(1, 2, 3);
/// assert_eq!(color.as_ref(), [2, 1, 3]);
/// assert_eq!((color.r(), color.g(), color.b(), color.w()), (1, 2, 3, 0));
/// ```
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
#[repr(transparent)]
pub struct LedPixelColorImpl<
    const N: usize,
    const R_ORDER: usize,
    const G_ORDER: usize,
    const B_ORDER: usize,
    const W_ORDER: usize,
>(pub(crate) [u8; N]);

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > LedPixelColor for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    const BPP: usize = N;

    #[inline]
    fn new_with_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new_with_rgbw(r, g, b, 0)
    }

    #[inline]
    fn new_with_rgbw(r: u8, g: u8, b: u8, w: u8) -> Self {
        let mut array = [0; N];
        if let Some(v) = array.get_mut(R_ORDER) {
            *v = r;
        }
        if let Some(v) = array.get_mut(G_ORDER) {
            *v = g;
        }
        if let Some(v) = array.get_mut(B_ORDER) {
            *v = b;
        }
        if let Some(v) = array.get_mut(W_ORDER) {
            *v = w;
        }
        Self(array)
    }

    #[inline]
    fn r(&self) -> u8 {
        self.0.get(R_ORDER).cloned().unwrap_or(0)
    }

    #[inline]
    fn g(&self) -> u8 {
        self.0.get(G_ORDER).cloned().unwrap_or(0)
    }

    #[inline]
    fn b(&self) -> u8 {
        self.0.get(B_ORDER).cloned().unwrap_or(0)
    }

    #[inline]
    fn w(&self) -> u8 {
        self.0.get(W_ORDER).cloned().unwrap_or(0)
    }
}

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > Default for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    /// Returns the black color (All LED OFF)
    #[inline]
    fn default() -> Self {
        Self([0; N])
    }
}

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > AsRef<[u8]> for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<
        const N: usize,
        const R_ORDER: usize,
        const G_ORDER: usize,
        const B_ORDER: usize,
        const W_ORDER: usize,
    > AsMut<[u8]> for LedPixelColorImpl<N, R_ORDER, G_ORDER, B_ORDER, W_ORDER>
{
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[test]
fn test_led_pixel_color_impl() {
    let color = LedPixelColorImpl::<3, 1, 0, 2, 255>::new_with_rgb(1, 2, 3);
    assert_eq!(color.0, [2, 1, 3]);
    assert_eq!(color.as_ref(), &color.0);
    assert_eq!((color.r(), color.g(), color.b(), color.w()), (1, 2, 3, 0));

    let color = LedPixelColorImpl::<3, 1, 0, 2, 255>::new_with_rgbw(1, 2, 3, 4);
    assert_eq!(color.0, [2, 1, 3]);
    assert_eq!(color.as_ref(), &color.0);
    assert_eq!((color.r(), color.g(), color.b(), color.w()), (1, 2, 3, 0));

    let color = LedPixelColorImpl::<4, 0, 1, 2, 3>::new_with_rgb(1, 2, 3);
    assert_eq!(color.0, [1, 2, 3, 0]);
    assert_eq!(color.as_ref(), &color.0);
    assert_eq!((color.r(), color.g(), color.b(), color.w()), (1, 2, 3, 0));

    let color = LedPixelColorImpl::<4, 0, 1, 2, 3>::new_with_rgbw(1, 2, 3, 4);
    assert_eq!(color.0, [1, 2, 3, 4]);
    assert_eq!(color.as_ref(), &color.0);
    assert_eq!((color.r(), color.g(), color.b(), color.w()), (1, 2, 3, 4));
}

#[test]
fn test_led_pixel_color_brightness() {
    let color = LedPixelColorImpl::<4, 0, 1, 2, 3>::new_with_rgbw(255, 128, 64, 32).brightness(128);
    assert_eq!(
        (color.r(), color.g(), color.b(), color.w()),
        (128, 64, 32, 16)
    );
}

/// 24bit GRB LED pixel color (Typical RGB LED (WS2812B/SK6812) pixel color)
pub type LedPixelColorGrb24 = LedPixelColorImpl<3, 1, 0, 2, 255>;
/// 32bit RGBW LED pixel color
pub type LedPixelColorRgbw32 = LedPixelColorImpl<4, 0, 1, 2, 3>;
/// 32bit GRBW LED pixel color
pub type LedPixelColorGrbw32 = LedPixelColorImpl<4, 1, 0, 2, 3>;
