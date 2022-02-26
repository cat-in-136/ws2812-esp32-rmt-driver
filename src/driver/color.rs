pub trait LedPixelColor:
    Ord + PartialOrd + Eq + PartialEq + Clone + AsRef<[u8]> + AsMut<[u8]>
{
    const BPP: usize;
    fn new_with_rgb(r: u8, g: u8, b: u8) -> Self;
    fn new_with_rgbw(r: u8, g: u8, b: u8, w: u8) -> Self;
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
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
        array.get_mut(R_ORDER).and_then(|v| Some(*v = r));
        array.get_mut(G_ORDER).and_then(|v| Some(*v = g));
        array.get_mut(B_ORDER).and_then(|v| Some(*v = b));
        array.get_mut(W_ORDER).and_then(|v| Some(*v = w));
        Self(array)
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

    let color = LedPixelColorImpl::<3, 1, 0, 2, 255>::new_with_rgbw(1, 2, 3, 4);
    assert_eq!(color.0, [2, 1, 3]);
    assert_eq!(color.as_ref(), &color.0);

    let color = LedPixelColorImpl::<4, 0, 1, 2, 3>::new_with_rgb(1, 2, 3);
    assert_eq!(color.0, [1, 2, 3, 0]);
    assert_eq!(color.as_ref(), &color.0);

    let color = LedPixelColorImpl::<4, 0, 1, 2, 3>::new_with_rgbw(1, 2, 3, 4);
    assert_eq!(color.0, [1, 2, 3, 4]);
    assert_eq!(color.as_ref(), &color.0);
}

pub type LedPixelColorGrb24 = LedPixelColorImpl<3, 1, 0, 2, 255>;
pub type LedPixelColorRgbw32 = LedPixelColorImpl<4, 0, 1, 2, 3>;
pub type LedPixelColorGrbw32 = LedPixelColorImpl<4, 1, 0, 2, 3>;
