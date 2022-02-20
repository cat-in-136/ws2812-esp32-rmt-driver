use crate::driver::Ws2812Esp32RmtDriver;
use smart_leds_trait::{SmartLedsWrite, RGB8};

use esp_idf_sys::*;

pub struct Ws2812Esp32Rmt {
    driver: Ws2812Esp32RmtDriver,
}

impl Ws2812Esp32Rmt {
    pub fn new(channel_num: u8, gpio_num: u32) -> Result<Self, EspError> {
        let driver = Ws2812Esp32RmtDriver::new(channel_num, gpio_num)?;
        Ok(Self { driver })
    }
}

impl SmartLedsWrite for Ws2812Esp32Rmt {
    type Error = EspError;
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
