use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::config::TransmitConfig;
use esp_idf_hal::rmt::{PinState, Pulse, RmtChannel, TxRmtDriver, VariableLengthSignal};
use esp_idf_hal::units::Hertz;
use esp_idf_sys::EspError;
use once_cell::sync::OnceCell;
use std::time::Duration;

static WS2812_ITEM_ENCODER: OnceCell<Ws2812Esp32RmtItemEncoder> = OnceCell::new();
const WS2812_TO0H_NS: Duration = Duration::from_nanos(400);
const WS2812_TO0L_NS: Duration = Duration::from_nanos(850);
const WS2812_TO1H_NS: Duration = Duration::from_nanos(800);
const WS2812_TO1L_NS: Duration = Duration::from_nanos(450);

#[repr(C)]
struct Ws2812Esp32RmtItemEncoder {
    bit0: (Pulse, Pulse),
    bit1: (Pulse, Pulse),
}

impl Ws2812Esp32RmtItemEncoder {
    fn new(clock_hz: Hertz) -> Result<Self, EspError> {
        let (t0h, t0l, t1h, t1l) = (
            Pulse::new_with_duration(clock_hz, PinState::High, &WS2812_TO0H_NS)?,
            Pulse::new_with_duration(clock_hz, PinState::Low, &WS2812_TO0L_NS)?,
            Pulse::new_with_duration(clock_hz, PinState::High, &WS2812_TO1H_NS)?,
            Pulse::new_with_duration(clock_hz, PinState::Low, &WS2812_TO1L_NS)?,
        );
        Ok(Self {
            bit0: (t0h, t0l),
            bit1: (t1h, t1l),
        })
    }

    fn encode_variable<T>(&self, src: T) -> Result<VariableLengthSignal, EspError>
    where
        T: Iterator<Item = u8>,
    {
        let mut s = VariableLengthSignal::new();

        for v in src {
            for i in 0..(u8::BITS as usize) {
                let bit_sig = if v & (1 << (7 - i)) != 0 {
                    self.bit1
                } else {
                    self.bit0
                };
                s.push([bit_sig.0, bit_sig.1].iter())?;
            }
        }

        Ok(s)
    }
}

/// WS2812 ESP32 RMT Driver error.
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Ws2812Esp32RmtDriverError(#[from] EspError);

/// WS2812 ESP32 RMT driver wrapper.
pub struct Ws2812Esp32RmtDriver<'d> {
    /// TxRMT driver.
    tx: TxRmtDriver<'d>,
}

impl<'d> Ws2812Esp32RmtDriver<'d> {
    /// Creates a WS2812 ESP32 RMT driver wrapper.
    ///
    /// RMT driver of `channel` shall be initialized and installed for `pin`.
    /// `channel` shall be different between different `pin`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RMT driver initialization failed.
    pub fn new<C: RmtChannel>(
        channel: impl Peripheral<P = C> + 'd,
        pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let config = TransmitConfig::new().clock_divider(1);
        let tx = TxRmtDriver::new(channel, pin, &config)?;

        let _encoder = WS2812_ITEM_ENCODER.get_or_try_init(|| {
            let clock_hz = tx.counter_clock()?;
            Ws2812Esp32RmtItemEncoder::new(clock_hz)
        })?;

        Ok(Self { tx })
    }

    /// Writes pixel data from the slice to the IO pin.
    ///
    /// Byte count per LED pixel and channel order is not handled by this method.
    /// The data has to be correctly laid out in the slice depending on the LED strip model.
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    pub fn write(&mut self, pixel_data: &[u8]) -> Result<(), Ws2812Esp32RmtDriverError> {
        if let Some(encoder) = WS2812_ITEM_ENCODER.get() {
            let signal = encoder.encode_variable(pixel_data.iter().cloned())?;
            self.tx.start_blocking(&signal)?;
        }
        Ok(())
    }
}
