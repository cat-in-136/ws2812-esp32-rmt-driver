#[derive(thiserror::Error, Debug)]
#[error("mock Ws2812Esp32RmtDriverError")]
pub struct Ws2812Esp32RmtDriverError;

pub struct Ws2812Esp32RmtDriver {
    pub grb_pixels: Option<Vec<u8>>,
    pub wait_tx_done: bool,
}

impl Ws2812Esp32RmtDriver {
    pub fn new(_channel_num: u8, _gpio_num: u32) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ok(Self {
            grb_pixels: None,
            wait_tx_done: true,
        })
    }

    pub fn write(&mut self, grb_pixels: &[u8]) -> Result<(), Ws2812Esp32RmtDriverError> {
        self.grb_pixels = Some(grb_pixels.to_vec());
        Ok(())
    }
}

#[test]
fn test_ws2812_esp32_rmt_driver_mock() {
    let sample_data: [u8; 6] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    let mut driver = Ws2812Esp32RmtDriver::new(0, 27).unwrap();
    assert_eq!(driver.grb_pixels, None);
    driver.write(&sample_data).unwrap();
    assert_eq!(driver.grb_pixels.unwrap(), &sample_data);
}
