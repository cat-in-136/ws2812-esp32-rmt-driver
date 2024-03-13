use std::marker::PhantomData;

/// WS2812 ESP32 RMT Driver error.
#[derive(thiserror::Error, Debug)]
#[error("mock Ws2812Esp32RmtDriverError")]
pub struct Ws2812Esp32RmtDriverError;

/// Mock of Low-level WS2812 ESP32 RMT driver.
///
/// If the target vendor does not equals to "espressif", this mock is used instead of genuine
/// Low-level WS2812 ESP32 RMT driver.
pub struct Ws2812Esp32RmtDriver<'d> {
    /// Pixel binary array to be written
    pub pixel_data: Option<Vec<u8>>,

    /// Dummy phantom to take care of lifetime
    phantom: PhantomData<&'d Option<Vec<u8>>>,
}

impl<'d> Ws2812Esp32RmtDriver<'d> {
    /// Creates a mock of `Ws2812Esp32RmtDriver`.
    pub fn new() -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ok(Self {
            pixel_data: None,
            phantom: Default::default(),
        })
    }

    /// Writes a pixel-byte sequence.
    pub fn write_blocking<'a, 'b, T>(
        &'a mut self,
        pixel_sequence: T,
    ) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        'b: 'a,
        T: Iterator<Item = u8> + Send + 'b,
    {
        self.pixel_data = Some(pixel_sequence.collect());
        Ok(())
    }

    /// Writes a pixel-byte sequence.
    pub fn write<'a, 'b, T>(
        &'a mut self,
        pixel_sequence: T,
    ) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        'b: 'a,
        T: Iterator<Item = u8> + Send + 'b,
    {
        self.pixel_data = Some(pixel_sequence.collect());
        Ok(())
    }
}

#[test]
fn test_ws2812_esp32_rmt_driver_mock() {
    let sample_data: [u8; 6] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    let mut driver = Ws2812Esp32RmtDriver::new().unwrap();
    assert_eq!(driver.pixel_data, None);
    driver.write_blocking(sample_data.iter().cloned()).unwrap();
    assert_eq!(driver.pixel_data.unwrap(), &sample_data);
}
