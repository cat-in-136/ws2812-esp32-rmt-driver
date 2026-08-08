//! WS2812 ESP32 RMT driver implementation using new RMT API (esp-idf-hal 0.46+)
#![cfg_attr(not(target_vendor = "espressif"), allow(dead_code))]

use core::error::Error;
use core::fmt;
use core::time::Duration;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_hal;
#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_sys::EspError;
use esp_idf_hal::{
    gpio::OutputPin,
    rmt::{config::TxChannelConfig, encoder::BytesEncoder, TxChannelDriver},
    units::Hertz,
};

#[cfg(target_vendor = "espressif")]
use esp_idf_hal::rmt::{
    config::TransmitConfig, encoder::BytesEncoderConfig, PinState, Pulse, Symbol, TxQueue,
};
#[cfg(target_vendor = "espressif")]
use esp_idf_sys::EspError;

/// Default RMT clock resolution used for pulse timing.
const DEFAULT_RMT_CLOCK_HZ: Hertz = Hertz(10_000_000); // 10 MHz

/// T0H duration time (0 code, high voltage time)
const WS2812_T0H_NS: Duration = Duration::from_nanos(400);
/// T0L duration time (0 code, low voltage time)
const WS2812_T0L_NS: Duration = Duration::from_nanos(850);
/// T1H duration time (1 code, high voltage time)
const WS2812_T1H_NS: Duration = Duration::from_nanos(800);
/// T1L duration time (1 code, low voltage time)
const WS2812_T1L_NS: Duration = Duration::from_nanos(450);

#[cfg(target_vendor = "espressif")]
fn make_bytes_encoder(
    clock_hz: Hertz,
    t0h: &Duration,
    t0l: &Duration,
    t1h: &Duration,
    t1l: &Duration,
) -> Result<BytesEncoder, EspError> {
    let config = BytesEncoderConfig {
        bit0: Symbol::new(
            Pulse::new_with_duration(clock_hz, PinState::High, *t0h)?,
            Pulse::new_with_duration(clock_hz, PinState::Low, *t0l)?,
        ),
        bit1: Symbol::new(
            Pulse::new_with_duration(clock_hz, PinState::High, *t1h)?,
            Pulse::new_with_duration(clock_hz, PinState::Low, *t1l)?,
        ),
        msb_first: true,
        ..Default::default()
    };
    BytesEncoder::with_config(&config)
}

#[cfg(not(target_vendor = "espressif"))]
fn make_bytes_encoder(
    _clock_hz: Hertz,
    _t0h: &Duration,
    _t0l: &Duration,
    _t1h: &Duration,
    _t1l: &Duration,
) -> Result<BytesEncoder, EspError> {
    Ok(BytesEncoder::default())
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
    /// This is a workaround function until `core::error::Error` added to `esp_sys::EspError`.
    pub fn source(&self) -> Option<&EspError> {
        Some(&self.source)
    }
}

impl Error for Ws2812Esp32RmtDriverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        #[cfg(feature = "std")]
        {
            Some(&self.source)
        }
        #[cfg(not(feature = "std"))]
        {
            None
        }
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

/// Builder for `Ws2812Esp32RmtDriver` (new API).
pub struct Ws2812Esp32RmtDriverBuilder<'d> {
    /// TxChannelDriver
    tx: TxChannelDriver<'d>,
    /// RMT clock resolution, taken from the TxChannelConfig.
    clock_hz: Hertz,
    /// BytesEncoder with WS2812 timing configuration.
    encoder: Option<BytesEncoder>,
}

impl<'d> Ws2812Esp32RmtDriverBuilder<'d> {
    /// Creates a new `Ws2812Esp32RmtDriverBuilder` with default config.
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let config = TxChannelConfig {
            resolution: DEFAULT_RMT_CLOCK_HZ,
            ..Default::default()
        };
        Self::new_with_config(pin, &config)
    }

    /// Creates a new `Ws2812Esp32RmtDriverBuilder` with a custom `TxChannelConfig`.
    pub fn new_with_config(
        pin: impl OutputPin + 'd,
        config: &TxChannelConfig,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let tx = TxChannelDriver::new(pin, config)?;
        Ok(Self {
            tx,
            clock_hz: config.resolution,
            encoder: None,
        })
    }

    /// Sets the encoder duration times.
    pub fn encoder_duration(
        mut self,
        t0h: &Duration,
        t0l: &Duration,
        t1h: &Duration,
        t1l: &Duration,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        self.encoder = Some(make_bytes_encoder(self.clock_hz, t0h, t0l, t1h, t1l)?);
        Ok(self)
    }

    /// Builds the `Ws2812Esp32RmtDriver`.
    pub fn build(self) -> Result<Ws2812Esp32RmtDriver<'d>, Ws2812Esp32RmtDriverError> {
        let encoder = if let Some(encoder) = self.encoder {
            encoder
        } else {
            make_bytes_encoder(
                self.clock_hz,
                &WS2812_T0H_NS,
                &WS2812_T0L_NS,
                &WS2812_T1H_NS,
                &WS2812_T1L_NS,
            )?
        };

        Ok(Ws2812Esp32RmtDriver {
            tx: self.tx,
            encoder,
            #[cfg(not(target_vendor = "espressif"))]
            pixel_data: None,
        })
    }
}

/// An in-progress non-blocking WS2812 transmission.
///
/// Dropping this value will block until the transmission is complete.
#[cfg(target_vendor = "espressif")]
pub struct Ws2812Esp32RmtTxQueue<'c, 'd> {
    queue: TxQueue<'c, 'd, &'c mut BytesEncoder>,
}

#[cfg(not(target_vendor = "espressif"))]
pub struct Ws2812Esp32RmtTxQueue<'c, 'd> {
    driver: &'c mut Ws2812Esp32RmtDriver<'d>,
}

/// WS2812 ESP32 RMT driver wrapper (new API).
#[allow(dead_code)]
pub struct Ws2812Esp32RmtDriver<'d> {
    /// TxChannelDriver
    tx: TxChannelDriver<'d>,
    /// BytesEncoder with WS2812 timing configuration.
    encoder: BytesEncoder,

    /// Pixel binary array to be written
    ///
    /// This is used only for non-espressif (mock) testing.
    #[cfg(not(target_vendor = "espressif"))]
    pub pixel_data: Option<Vec<u8>>,
}

impl<'d> Ws2812Esp32RmtDriver<'d> {
    /// Creates a WS2812 ESP32 RMT driver wrapper with default config.
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ws2812Esp32RmtDriverBuilder::new(pin)?.build()
    }

    /// Creates a WS2812 ESP32 RMT driver wrapper with a custom `TxChannelConfig`.
    pub fn new_with_config(
        pin: impl OutputPin + 'd,
        config: &TxChannelConfig,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ws2812Esp32RmtDriverBuilder::new_with_config(pin, config)?.build()
    }

    /// Writes pixel data from a pixel-byte iterator to the IO pin.
    ///
    /// Byte count per LED pixel and channel order is not handled by this method.
    /// The pixel data sequence has to be correctly laid out depending on the LED strip model.
    ///
    /// # Arguments
    ///
    /// * `pixel_sequence` - Iterator of pixel bytes (backward compatible signature)
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    pub fn write_blocking<T>(&mut self, pixel_sequence: T) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        T: Iterator<Item = u8>,
    {
        // Collect into Vec and call write_blocking_slice
        let data: Vec<u8> = pixel_sequence.collect();
        self.write_blocking_slice(&data)
    }

    /// Writes pixel data from a slice to the IO pin (high-performance method).
    ///
    /// This method avoids the Vec allocation that `write_blocking` with an iterator may cause.
    ///
    /// # Arguments
    ///
    /// * `pixel_data` - Slice of pixel bytes
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    pub fn write_blocking_slice(
        &mut self,
        pixel_data: &[u8],
    ) -> Result<(), Ws2812Esp32RmtDriverError> {
        if pixel_data.is_empty() {
            return Ok(()); // No-op for empty data
        }
        let mut queue = self.queue();
        queue.push_blocking(pixel_data)?;
        Ok(())
    }

    /// Creates a new queue to transmit multiple symbols.
    ///
    /// This is mostly useful for non-blocking transmission, but it can also be used
    /// for blocking transmission if then the caller wants to reuse the same queue for
    /// multiple transmissions.
    ///
    /// Note: Dropping the queue will block until the transmission is complete,
    /// so be careful to not drop the queue prematurely.
    pub fn queue<'a>(&'a mut self) -> Ws2812Esp32RmtTxQueue<'a, 'd> {
        #[cfg(target_vendor = "espressif")]
        {
            let queue = self.tx.queue(core::iter::once(&mut self.encoder));
            Ws2812Esp32RmtTxQueue { queue }
        }
        #[cfg(not(target_vendor = "espressif"))]
        {
            Ws2812Esp32RmtTxQueue { driver: self }
        }
    }
}

impl<'c, 'd> Ws2812Esp32RmtTxQueue<'c, 'd> {
    /// Writes pixel data to the IO pin, using a pre-made queue, without blocking.
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred, for example if the data cannot
    /// be pushed to the transmission queue without blocking.
    pub fn push(&mut self, pixel: &[u8]) -> Result<(), Ws2812Esp32RmtDriverError> {
        if pixel.is_empty() {
            return Ok(()); // No-op for empty data
        }
        #[cfg(target_vendor = "espressif")]
        {
            let config = TransmitConfig {
                queue_non_blocking: true,
                ..Default::default()
            };
            self.queue.push(pixel, &config)?;
        }
        #[cfg(not(target_vendor = "espressif"))]
        {
            self.driver.pixel_data = Some(pixel.to_vec());
        }
        Ok(())
    }

    /// Writes pixel data to the IO pin, using a pre-made queue, blocking.
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    pub fn push_blocking(&mut self, pixel: &[u8]) -> Result<(), Ws2812Esp32RmtDriverError> {
        if pixel.is_empty() {
            return Ok(()); // No-op for empty data
        }
        #[cfg(target_vendor = "espressif")]
        self.queue.push(pixel, &TransmitConfig::default())?;
        #[cfg(not(target_vendor = "espressif"))]
        {
            self.driver.pixel_data = Some(pixel.to_vec());
        }
        Ok(())
    }
}
