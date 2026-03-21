#![cfg_attr(not(target_vendor = "espressif"), allow(dead_code))]

use core::convert::From;
use core::error::Error;
use core::fmt;
use core::time::Duration;

#[cfg(not(target_vendor = "espressif"))]
use core::marker::PhantomData;

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_hal;
#[cfg(target_vendor = "espressif")]
use esp_idf_hal::rmt::{
    config::TransmitConfig, encoder::BytesEncoderConfig, PinState, Pulse, Symbol,
};
use esp_idf_hal::{
    gpio::OutputPin,
    rmt::{config::TxChannelConfig, encoder::BytesEncoder, TxChannelDriver},
    units::Hertz,
};

#[cfg(not(target_vendor = "espressif"))]
use crate::mock::esp_idf_sys;
use esp_idf_sys::EspError;

/// RMT clock resolution used for pulse timing.
/// TODO: Is it ok to hardcode this? -- this matches HAL examples.
const RMT_CLOCK_HZ: Hertz = Hertz(10_000_000); // 10 MHz

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
    Ok(BytesEncoder {})
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

/// Builder for `Ws2812Esp32RmtDriver`.
///
/// # Examples
///
///
/// ```
/// # #[cfg(not(target_vendor = "espressif"))]
/// # use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
/// #
/// # use core::time::Duration;
/// # use esp_idf_hal::peripherals::Peripherals;
/// # use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriverBuilder;
/// #
/// # let peripherals = Peripherals::take().unwrap();
/// # let led_pin = peripherals.pins.gpio27;
///
/// // WS2812B timing parameters.
/// const WS2812_T0H_NS: Duration = Duration::from_nanos(400);
/// const WS2812_T0L_NS: Duration = Duration::from_nanos(850);
/// const WS2812_T1H_NS: Duration = Duration::from_nanos(800);
/// const WS2812_T1L_NS: Duration = Duration::from_nanos(450);
///
/// let driver = Ws2812Esp32RmtDriverBuilder::new(led_pin).unwrap()
///    .encoder_duration(&WS2812_T0H_NS, &WS2812_T0L_NS, &WS2812_T1H_NS, &WS2812_T1L_NS).unwrap()
///    .build().unwrap();
/// ```
pub struct Ws2812Esp32RmtDriverBuilder<'d> {
    /// TxRMT driver.
    tx: TxChannelDriver<'d>,
    /// BytesEncoder with WS2812 timing configuration.
    encoder: Option<BytesEncoder>,
}

impl<'d> Ws2812Esp32RmtDriverBuilder<'d> {
    /// Creates a new `Ws2812Esp32RmtDriverBuilder`.
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let config = TxChannelConfig {
            resolution: RMT_CLOCK_HZ,
            ..Default::default()
        };
        let tx = TxChannelDriver::new(pin, &config)?;
        Ok(Self { tx, encoder: None })
    }

    /// Sets the encoder duration times.
    ///
    /// # Arguments
    ///
    /// * `t0h` - T0H duration time (0 code, high voltage time)
    /// * `t0l` - T0L duration time (0 code, low voltage time)
    /// * `t1h` - T1H duration time (1 code, high voltage time)
    /// * `t1l` - T1L duration time (1 code, low voltage time)
    ///
    /// Note: the clock resolution is fixed at 10 MHz.
    ///
    /// # Errors
    ///
    /// Returns an error if the encoder initialization failed.
    pub fn encoder_duration(
        mut self,
        t0h: &Duration,
        t0l: &Duration,
        t1h: &Duration,
        t1l: &Duration,
    ) -> Result<Self, Ws2812Esp32RmtDriverError> {
        self.encoder = Some(make_bytes_encoder(RMT_CLOCK_HZ, t0h, t0l, t1h, t1l)?);
        Ok(self)
    }

    /// Builds the `Ws2812Esp32RmtDriver`.
    pub fn build(self) -> Result<Ws2812Esp32RmtDriver<'d>, Ws2812Esp32RmtDriverError> {
        let encoder = if let Some(encoder) = self.encoder {
            encoder
        } else {
            make_bytes_encoder(
                RMT_CLOCK_HZ,
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
            #[cfg(not(target_vendor = "espressif"))]
            phantom: Default::default(),
        })
    }
}

/// WS2812 ESP32 RMT driver wrapper.
///
/// # Examples
///
/// ```
/// # #[cfg(not(target_vendor = "espressif"))]
/// # use ws2812_esp32_rmt_driver::mock::esp_idf_hal;
/// #
/// use esp_idf_hal::peripherals::Peripherals;
/// use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;
/// use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
///
/// let peripherals = Peripherals::take().unwrap();
/// let led_pin = peripherals.pins.gpio27;
/// let mut driver = Ws2812Esp32RmtDriver::new(led_pin).unwrap();
///
/// // Single LED with RED color.
/// let red = LedPixelColorGrb24::new_with_rgb(30, 0, 0);
/// let pixel: [u8; 3] = red.as_ref().try_into().unwrap();
/// assert_eq!(pixel, [0, 30, 0]);
///
/// driver.write_blocking(core::iter::once(pixel.as_ref())).unwrap();
/// ```
pub struct Ws2812Esp32RmtDriver<'d> {
    /// TxChannelDriver
    tx: TxChannelDriver<'d>,
    /// BytesEncoder with WS2812 timing configuration.
    encoder: BytesEncoder,

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
    /// # Errors
    ///
    /// Returns an error if the RMT driver initialization failed.
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self, Ws2812Esp32RmtDriverError> {
        Ws2812Esp32RmtDriverBuilder::new(pin)?.build()
    }

    /// Writes pixel data from a pixel-byte sequence to the IO pin.
    ///
    /// Byte count per LED pixel and channel order is not handled by this method.
    /// The pixel data sequence has to be correctly laid out depending on the LED strip model.
    ///
    /// # Errors
    ///
    /// Returns an error if an RMT driver error occurred.
    pub fn write_blocking<S, T>(
        &mut self,
        pixel_sequence: T,
    ) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        S: AsRef<[u8]>,
        T: Iterator<Item = S>,
    {
        #[cfg(target_vendor = "espressif")]
        {
            self.tx.send_iter(
                core::iter::once(&mut self.encoder),
                pixel_sequence,
                &TransmitConfig::default(),
            )?;
        }
        #[cfg(not(target_vendor = "espressif"))]
        {
            self.pixel_data = Some(pixel_sequence.flat_map(|s| s.as_ref().to_vec()).collect());
        }
        Ok(())
    }
}
