use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::config::TransmitConfig;
use esp_idf_hal::rmt::{PinState, Pulse, PulseTicks, RmtChannel, Symbol, TxRmtDriver};
use esp_idf_hal::units::Hertz;
use esp_idf_sys::EspError;
use std::time::Duration;

/// T0H duration time (0 code, high voltage time)
const WS2812_T0H_NS: Duration = Duration::from_nanos(400);
/// T0L duration time (0 code, low voltage time)
const WS2812_T0L_NS: Duration = Duration::from_nanos(850);
/// T1H duration time (1 code, high voltage time)
const WS2812_T1H_NS: Duration = Duration::from_nanos(800);
/// T1L duration time (1 code, low voltage time)
const WS2812_T1L_NS: Duration = Duration::from_nanos(450);

/// Converter to a sequence of RMT items.
#[repr(C)]
struct Ws2812Esp32RmtItemEncoder {
    tick_t0_h_ns: PulseTicks,
    tick_t0_l_ns: PulseTicks,
    tick_t1_h_ns: PulseTicks,
    tick_t1_l_ns: PulseTicks,
}

impl Ws2812Esp32RmtItemEncoder {
    /// Creates a new encoder with the given clock frequency.
    ///
    /// # Arguments
    ///
    /// * `clock_hz` - The clock frequency.
    ///
    /// # Errors
    ///
    /// Returns an error if the clock frequency is invalid or if the RMT item encoder cannot be created.
    fn new(clock_hz: Hertz) -> Result<Self, EspError> {
        let tick_t0_h_ns = PulseTicks::new_with_duration(clock_hz, &WS2812_T0H_NS)?;
        let tick_t0_l_ns = PulseTicks::new_with_duration(clock_hz, &WS2812_T0L_NS)?;
        let tick_t1_h_ns = PulseTicks::new_with_duration(clock_hz, &WS2812_T1H_NS)?;
        let tick_t1_l_ns = PulseTicks::new_with_duration(clock_hz, &WS2812_T1L_NS)?;

        Ok(Self {
            tick_t0_h_ns,
            tick_t0_l_ns,
            tick_t1_h_ns,
            tick_t1_l_ns,
        })
    }

    /// Encodes a block of data as a sequence of RMT items.
    ///
    /// # Arguments
    ///
    /// * `src` - The block of data to encode.
    ///
    /// # Returns
    ///
    /// An iterator over the RMT items that represent the encoded data.
    #[inline]
    fn encode_iter<'a, 'b, T>(&'a self, src: T) -> impl Iterator<Item = Symbol> + Send + 'a
    where
        'b: 'a,
        T: Iterator<Item = u8> + Send + 'b,
    {
        let (t0h, t0l, t1h, t1l) = (
            Pulse::new(PinState::High, self.tick_t0_h_ns),
            Pulse::new(PinState::Low, self.tick_t0_l_ns),
            Pulse::new(PinState::High, self.tick_t1_h_ns),
            Pulse::new(PinState::Low, self.tick_t1_l_ns),
        );

        src.flat_map(move |v| {
            (0..(u8::BITS as usize)).map(move |i| {
                if v & (1 << (7 - i)) != 0 {
                    Symbol::new(t1h, t1l)
                } else {
                    Symbol::new(t0h, t0l)
                }
            })
        })
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
    /// `u8`-to-`rmt_item32_t` Encoder
    encoder: Ws2812Esp32RmtItemEncoder,
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

        let clock_hz = tx.counter_clock()?;
        let encoder = Ws2812Esp32RmtItemEncoder::new(clock_hz)?;

        Ok(Self { tx, encoder })
    }

    /// Writes pixel data from a pixel-byte sequence to the IO pin.
    ///
    /// Byte count per LED pixel and channel order is not handled by this method.
    /// The pixel data sequence has to be correctly laid out depending on the LED strip model.
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    ///
    /// # Warning
    ///
    /// Iteration of `pixel_sequence` happens inside an interrupt handler so beware of side-effects
    /// that don't work in interrupt handlers.
    /// See [esp_idf_hal::rmt::TxRmtDriver#start_iter_blocking()] for details.
    pub fn write_blocking<'a, 'b, T>(
        &'a mut self,
        pixel_sequence: T,
    ) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        'b: 'a,
        T: Iterator<Item = u8> + Send + 'b,
    {
        let signal = self.encoder.encode_iter(pixel_sequence);
        self.tx.start_iter_blocking(signal)?;
        Ok(())
    }

    /// Writes pixel data from a pixel-byte sequence to the IO pin.
    ///
    /// Byte count per LED pixel and channel order is not handled by this method.
    /// The pixel data sequence has to be correctly laid out depending on the LED strip model.
    ///
    /// Note that this requires `pixel_sequence` to be [`Box`]ed for an allocation free version see [`Self::write_blocking`].
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    ///
    /// # Warning
    ///
    /// Iteration of `pixel_sequence` happens inside an interrupt handler so beware of side-effects
    /// that don't work in interrupt handlers.
    /// See [esp_idf_hal::rmt::TxRmtDriver#start_iter()] for details.
    #[cfg(feature = "unstable")]
    pub fn write<'b, T>(
        &'static mut self,
        pixel_sequence: T,
    ) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        T: Iterator<Item = u8> + Send + 'static,
    {
        let signal = self.encoder.encode_iter(pixel_sequence);
        self.tx.start_iter(signal)?;
        Ok(())
    }
}
