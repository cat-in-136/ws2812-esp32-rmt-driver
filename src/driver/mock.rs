#[derive(thiserror::Error, Debug)]
#[error("mock Ws2812Esp32RmtDriverError")]
pub struct  Ws2812Esp32RmtDriverError;

pub struct Ws2812Esp32RmtDriver {
    grb_pixels: Option<Vec<u8>>,
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

    pub fn grb_pixels_debug(&self) -> Option<&[u8]> {
        self.grb_pixels.as_ref().map(|v| v.as_slice())
    }
}
