/// WS2812 ESP32 RMT Driver error.
#[derive(thiserror::Error, Debug)]
#[error("mock Ws2812Esp32RmtDriverError")]
pub struct Ws2812Esp32RmtDriverError;

/// Mock of Low-level WS2812 ESP32 RMT driver.
///
/// If the target vendor does not equals to "espressif", this mock is used instead of genuine
/// Low-level WS2812 ESP32 RMT driver.
pub struct Ws2812Esp32RmtDriver {
    /// Pixel binary array to be written
    pub pixel_data: Option<Vec<u8>>,
}

impl Ws2812Esp32RmtDriver {
    /// Creates a mock of `Ws2812Esp32RmtDriver`.
    /// All arguments shall be ignored and always returns `Ok(_)`.
    pub fn new(_channel_num: u8, _gpio_num: u32) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ok(Self { pixel_data: None })
    }

    /// Writes GRB pixel binary slice.
    pub fn write(&mut self, pixel_data: &[u8]) -> Result<(), Ws2812Esp32RmtDriverError> {
        self.pixel_data = Some(pixel_data.to_vec());
        Ok(())
    }
}

#[test]
fn test_ws2812_esp32_rmt_driver_mock() {
    let sample_data: [u8; 6] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    let mut driver = Ws2812Esp32RmtDriver::new(0, 27).unwrap();
    assert_eq!(driver.pixel_data, None);
    driver.write(&sample_data).unwrap();
    assert_eq!(driver.pixel_data.unwrap(), &sample_data);
}
