use crate::driver::{Ws2812Esp32RmtDriver, Ws2812Esp32RmtDriverError};
use smart_leds_trait::{SmartLedsWrite, RGB8};

pub struct Ws2812Esp32Rmt {
    driver: Ws2812Esp32RmtDriver,
}

impl Ws2812Esp32Rmt {
    pub fn new(channel_num: u8, gpio_num: u32) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let driver = Ws2812Esp32RmtDriver::new(channel_num, gpio_num)?;
        Ok(Self { driver })
    }
}

impl SmartLedsWrite for Ws2812Esp32Rmt {
    type Error = Ws2812Esp32RmtDriverError;
    type Color = RGB8;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        let grb = iterator
            .flat_map(|v| {
                let rgb = v.into();
                [rgb.g, rgb.r, rgb.b]
            })
            .collect::<Vec<_>>();

        self.driver.write(&grb)
    }
}

#[test]
#[cfg(not(target_vendor = "espressif"))]
fn test_ws2812_esp32_rmt_smart_leds() {
    let sample_data = [RGB8::new(0x00, 0x01, 0x02), RGB8::new(0x03, 0x04, 0x05)];
    let expected_values: [u8; 6] = [0x01, 0x00, 0x02, 0x04, 0x03, 0x05];
    let mut ws2812 = Ws2812Esp32Rmt::new(0, 27).unwrap();
    ws2812.write(sample_data.iter().cloned()).unwrap();
    assert_eq!(ws2812.driver.grb_pixels.unwrap(), &expected_values);
}