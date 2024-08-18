#![cfg_attr(not(target_vendor = "espressif"), allow(dead_code))]

use core::convert::From;
use core::fmt;
use core::time::Duration;

#[cfg(not(target_vendor = "espressif"))]
use core::marker::PhantomData;

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_hal;
use esp_idf_hal::{
    gpio::OutputPin,
    peripheral::Peripheral,
    rmt::{config::TransmitConfig, RmtChannel, TxRmtDriver},
};
#[cfg(target_vendor = "espressif")]
use esp_idf_hal::{
    rmt::{PinState, Pulse, Symbol},
    units::Hertz,
};

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_sys;
use esp_idf_sys::EspError;

#[cfg(feature = "std")]
use std::error::Error;

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
#[cfg(target_vendor = "espressif")]
struct Ws2812Esp32RmtItemEncoder {
    /// The RMT item that represents a 0 code.
    bit0: Symbol,
    /// The RMT item that represents a 1 code.
    bit1: Symbol,
}

#[cfg(target_vendor = "espressif")]
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
        let (bit0, bit1) = (
            Symbol::new(
                Pulse::new_with_duration(clock_hz, PinState::High, &WS2812_T0H_NS)?,
                Pulse::new_with_duration(clock_hz, PinState::Low, &WS2812_T0L_NS)?,
            ),
            Symbol::new(
                Pulse::new_with_duration(clock_hz, PinState::High, &WS2812_T1H_NS)?,
                Pulse::new_with_duration(clock_hz, PinState::Low, &WS2812_T1L_NS)?,
            ),
        );

        Ok(Self { bit0, bit1 })
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
        src.flat_map(move |v| {
            (0..(u8::BITS as usize)).map(move |i| {
                if v & (1 << (7 - i)) != 0 {
                    self.bit1
                } else {
                    self.bit0
                }
            })
        })
    }
}

/// WS2812 ESP32 RMT Driver error.
#[derive(Debug)]
#[repr(transparent)]
pub struct Ws2812Esp32RmtDriverError {
    source: EspError,
}

#[cfg(not(feature = "std"))]
impl Ws2812Esp32RmtDriverError {
    /// The `EspError` source of this error, if any.
    ///
    /// This is a workaround function until `core::error::Error` added.
    pub fn source(&self) -> Option<&EspError> {
        Some(&self.source)
    }
}

#[cfg(feature = "std")]
impl Error for Ws2812Esp32RmtDriverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for Ws2812Esp32RmtDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

impl From<EspError> for Ws2812Esp32RmtDriverError {
    fn from(source: EspError) -> Self {
        Self { source }
    }
}

/// WS2812 ESP32 RMT driver wrapper.
///
/// # Examples
///
/// ```
/// #[cfg(not(target_vendor = "espressif"))]
/// use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
///
/// use esp_idf_hal::peripherals::Peripherals;
/// use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;
/// use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
///
/// let peripherals = Peripherals::take().unwrap();
/// let led_pin = peripherals.pins.gpio27;
/// let channel = peripherals.rmt.channel0;
/// let mut driver = Ws2812Esp32RmtDriver::new(channel, led_pin).unwrap();
///
/// // Single LED with RED color.
/// let red = LedPixelColorGrb24::new_with_rgb(30, 0, 0);
/// let pixel: [u8; 3] = red.as_ref().try_into().unwrap();
/// assert_eq!(pixel, [0, 30, 0]);
///
/// driver.write_blocking(pixel.clone().into_iter()).unwrap();
/// ```
pub struct Ws2812Esp32RmtDriver<'d> {
    /// TxRMT driver.
    tx: TxRmtDriver<'d>,
    /// `u8`-to-`rmt_item32_t` Encoder
    #[cfg(target_vendor = "espressif")]
    encoder: Ws2812Esp32RmtItemEncoder,

    /// Pixel binary array to be written
    ///
    /// If the target vendor does not equals to "espressif", pixel data is written into this
    /// instead of genuine encoder.
    #[cfg(not(target_vendor = "espressif"))]
    pub pixel_data: Option<Vec<u8>>,
    /// Dummy phantom to take care of lifetime for `pixel_data`.
    #[cfg(not(target_vendor = "espressif"))]
    phantom: PhantomData<&'d Option<Vec<u8>>>,
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
        #[cfg(target_vendor = "espressif")]
        {
            let config = TransmitConfig::new().clock_divider(1);
            let tx = TxRmtDriver::new(channel, pin, &config)?;

            let clock_hz = tx.counter_clock()?;
            let encoder = Ws2812Esp32RmtItemEncoder::new(clock_hz)?;

            Ok(Self { tx, encoder })
        }
        #[cfg(not(target_vendor = "espressif"))] // Mock implement
        {
            let config = TransmitConfig::new();
            let tx = TxRmtDriver::new(channel, pin, &config)?;
            Ok(Self {
                tx,
                pixel_data: None,
                phantom: Default::default(),
            })
        }
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
        #[cfg(target_vendor = "espressif")]
        {
            let signal = self.encoder.encode_iter(pixel_sequence);
            self.tx.start_iter_blocking(signal)?;
        }
        #[cfg(not(target_vendor = "espressif"))]
        {
            self.pixel_data = Some(pixel_sequence.collect());
        }
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
    #[cfg(feature = "alloc")]
    pub fn write<'b, T>(
        &'static mut self,
        pixel_sequence: T,
    ) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        T: Iterator<Item = u8> + Send + 'static,
    {
        #[cfg(target_vendor = "espressif")]
        {
            let signal = self.encoder.encode_iter(pixel_sequence);
            self.tx.start_iter(signal)?;
        }
        #[cfg(not(target_vendor = "espressif"))]
        {
            self.pixel_data = Some(pixel_sequence.collect());
        }
        Ok(())
    }
}
