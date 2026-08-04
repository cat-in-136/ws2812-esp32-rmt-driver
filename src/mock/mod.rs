//! Mock modules for local testing

/// Mock module for `esp_idf_hal`
pub mod esp_idf_hal {
    pub use super::esp_idf_sys as sys;

    /// Mock module for `esp_idf_hal::gpio`
    pub mod gpio {
        use paste::paste;

        /// Mock trait for `esp_idf_hal::gpio::OutputPin`.
        pub trait OutputPin {}

        macro_rules! define_pins_struct {
            ($($num:expr),*) => {
                paste! {
                    /// Mock struct for `esp_idf_hal::gpio::Pins`.
                    #[derive(Debug, Default)]
                    pub struct Pins {
                        $(
                            pub [<gpio $num>]: [<Gpio $num>],
                        )*
                    }
                }
            }
        }
        define_pins_struct!(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48
        );

        impl Pins {
            pub(super) fn new() -> Self {
                Default::default()
            }
        }

        macro_rules! define_gpio_structs {
            ($($num:expr),*) => {
                paste! {
                    $(
                        #[doc = concat!("Mock struct for `esp_idf_hal::gpio::Gpio", stringify!($num) ,"`")]
                        #[derive(Debug, Default)]
                        pub struct [<Gpio $num>] {}

                        //impl [<Gpio $num>] {
                        //    pub(super) fn new() -> Self {
                        //        Self {}
                        //    }
                        //}

                        impl OutputPin for [<Gpio $num>] {}
                    )*
                }
            };
        }
        define_gpio_structs!(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48
        );
    }

    /// Mock module for `esp_idf_hal::peripherals`
    pub mod peripherals {
        /// Mock struct for `esp_idf_hal::peripherals::Peripherals`
        pub struct Peripherals {
            pub pins: super::gpio::Pins,
            #[cfg(feature = "rmt-legacy")]
            pub rmt: super::rmt::RMT,
        }

        impl Peripherals {
            pub fn take() -> Result<Self, super::sys::EspError> {
                Ok(Self::new())
            }

            // Create `Peripherals` instance.
            //
            // This function shall not used usually because
            // the original `esp_idf_hal::peripherals::Peripherals::new()` is unsafe,
            // and `take()` should be used instead.
            pub fn new() -> Self {
                Self {
                    pins: super::gpio::Pins::new(),
                    #[cfg(feature = "rmt-legacy")]
                    rmt: super::rmt::RMT::new(),
                }
            }
        }
    }

    /// Mock module for `esp_idf_hal::rmt`
    #[cfg(feature = "rmt-legacy")]
    pub mod rmt {
        use super::gpio::OutputPin;
        use super::sys::EspError;
        use super::units::Hertz;
        use config::TransmitConfig;
        use core::marker::PhantomData;
        use paste::paste;

        macro_rules! define_channel_structs {
            ($($num:expr),*) => {
                paste! {
                    $(
                        #[doc = concat!("Mock struct for `esp_idf_hal::rmt::CHANNEL", stringify!($num) ,"`")]
                        #[derive(Debug, Default)]
                        pub struct [<CHANNEL $num>] {}

                        impl [<CHANNEL $num>] {
                            pub fn new() -> Self {
                                Self {}
                            }
                        }

                        impl RmtChannel for [<CHANNEL $num>] {}
                    )*
                }
            };
        }
        define_channel_structs!(0, 1, 2, 3, 4, 5, 6, 7);

        /// mock struct for `esp_idf_hal::rmt::RMT`
        #[derive(Debug, Default)]
        pub struct RMT {
            pub channel0: CHANNEL0,
            pub channel1: CHANNEL1,
            pub channel2: CHANNEL2,
            pub channel3: CHANNEL3,
            pub channel4: CHANNEL4,
            pub channel5: CHANNEL5,
            pub channel6: CHANNEL6,
            pub channel7: CHANNEL7,
        }

        impl RMT {
            pub fn new() -> Self {
                Default::default()
            }
        }

        /// Mock trait fo `esp_idf_hal::rmt::RmtChannel`
        pub trait RmtChannel {}

        //pub type RmtTransmitConfig = config::TransmitConfig;

        /// Mock module for `esp_idf_hal::rmt::TxRmtDriver`
        pub struct TxRmtDriver<'d> {
            _p: PhantomData<&'d mut ()>,
        }

        impl<'d> TxRmtDriver<'d> {
            /// Initialize the mock of `TxRmtDriver`.
            /// No argument is used in this mock.
            pub fn new<C: RmtChannel + 'd>(
                _channel: C,
                _pin: impl OutputPin + 'd,
                _config: &TransmitConfig,
            ) -> Result<Self, EspError> {
                Ok(Self { _p: PhantomData })
            }

            pub fn counter_clock(&self) -> Result<Hertz, EspError> {
                let ticks_hz: u32 = 80000000; // 80MHz
                Ok(Hertz(ticks_hz))
            }
        }

        /// Mock module for `esp_idf_hal::rmt::config`
        pub mod config {
            /// Mock struct for `esp_idf_hal::rmt::config::TransmitConfig`
            #[derive(Debug, Clone)]
            pub struct TransmitConfig {
                pub clock_divider: u8,
                pub mem_block_num: u8,
                // Other parameters are omitted
            }

            impl TransmitConfig {
                pub fn new() -> Self {
                    Self {
                        mem_block_num: 1,
                        clock_divider: 80,
                    }
                }
                #[must_use]
                pub fn clock_divider(mut self, divider: u8) -> Self {
                    self.clock_divider = divider;
                    self
                }
                #[must_use]
                pub fn mem_block_num(mut self, mem_block_num: u8) -> Self {
                    self.mem_block_num = mem_block_num;
                    self
                }
            }
        }
    }

    /// Mock module for `esp_idf_hal::rmt`
    #[cfg(not(feature = "rmt-legacy"))]
    pub mod rmt {
        use super::gpio::OutputPin;
        use super::units::Hertz;
        use config::TxChannelConfig;
        use core::marker::PhantomData;

        /// Mock enum for `esp_idf_hal::rmt::PinState`
        #[derive(Debug, Clone, Copy)]
        pub enum PinState {
            High,
            Low,
        }

        /// Mock struct for `esp_idf_hal::rmt::TxChannelDriver`
        pub struct TxChannelDriver<'d> {
            _p: PhantomData<&'d mut ()>,
        }

        impl<'d> TxChannelDriver<'d> {
            /// Initialize the mock of `TxChannelDriver`.
            pub fn new(
                _pin: impl OutputPin + 'd,
                _config: &TxChannelConfig,
            ) -> Result<Self, crate::mock::esp_idf_sys::EspError> {
                Ok(Self { _p: PhantomData })
            }
        }

        /// Mock module for `esp_idf_hal::rmt::config`
        pub mod config {
            use super::super::units::Hertz;

            /// Mock struct for `esp_idf_hal::rmt::config::TxChannelConfig`
            #[derive(Debug, Clone, Default)]
            pub struct TxChannelConfig {
                pub resolution: Hertz,
                pub interrupt_priority: u8,
                pub memory_access: MemoryAccess,
                pub transaction_queue_depth: u8,
                pub invert_out: bool,
                pub io_loop_back: bool,
                pub io_od_mode: bool,
                pub allow_pd: bool,
            }

            impl TxChannelConfig {
                pub fn new() -> Self {
                    Self {
                        resolution: Hertz(10_000_000),
                        ..Default::default()
                    }
                }
            }

            #[derive(Debug, Clone, Default)]
            pub struct MemoryAccess {
                pub symbols: usize,
                pub is_direct: bool,
            }

            impl MemoryAccess {
                pub fn symbols(&self) -> usize {
                    self.symbols
                }
                pub fn is_direct(&self) -> bool {
                    self.is_direct
                }
            }

            /// Mock struct for `esp_idf_hal::rmt::config::TransmitConfig`
            #[derive(Debug, Clone, Default)]
            pub struct TransmitConfig {
                pub loop_count: i32,
                pub eot_level: bool,
                pub queue_non_blocking: bool,
            }

            impl TransmitConfig {
                pub fn new() -> Self {
                    Default::default()
                }
            }
        }

        /// Mock module for `esp_idf_hal::rmt::encoder`
        pub mod encoder {
            use core::marker::PhantomData;

            /// Mock struct for `esp_idf_hal::rmt::encoder::BytesEncoder`
            #[derive(Debug, Default)]
            pub struct BytesEncoder {
                _marker: PhantomData<()>,
            }

            impl BytesEncoder {
                pub fn with_config(
                    _config: &BytesEncoderConfig,
                ) -> Result<Self, crate::mock::esp_idf_sys::EspError> {
                    Ok(Self {
                        _marker: PhantomData,
                    })
                }
            }

            /// Mock struct for `esp_idf_hal::rmt::encoder::BytesEncoderConfig`
            #[derive(Debug, Default)]
            pub struct BytesEncoderConfig {
                pub bit0: super::Symbol,
                pub bit1: super::Symbol,
                pub msb_first: bool,
            }

            /// Mock trait for `Encoder`
            pub trait Encoder {
                type Item;
            }

            /// Mock trait for `RawEncoder`
            pub trait RawEncoder {
                type Item;
                fn handle(&mut self) -> *mut ();
            }

            /// Convert encoder to raw
            pub fn into_raw<E: Encoder>(_encoder: E) -> *mut () {
                core::ptr::null_mut()
            }
        }

        /// Mock struct for `esp_idf_hal::rmt::TxQueue`
        pub struct TxQueue<'c, 'd, E> {
            _marker: PhantomData<(&'c mut (), &'d mut (), E)>,
        }

        impl<'c, 'd, E> TxQueue<'c, 'd, E> {
            pub fn new() -> Self {
                Self {
                    _marker: PhantomData,
                }
            }

            pub fn push(
                &mut self,
                signal: &[u8],
                _config: &config::TransmitConfig,
            ) -> Result<(), crate::mock::esp_idf_sys::EspError> {
                if signal.is_empty() {
                    // Empty signals are a no-op in the mock (consistent with driver behavior)
                    return Ok(());
                }
                Ok(())
            }
        }

        /// Mock struct for `esp_idf_hal::rmt::Symbol`
        #[derive(Debug, Clone, Copy, Default)]
        pub struct Symbol {
            pub level0: bool,
            pub duration0: u16,
            pub level1: bool,
            pub duration1: u16,
        }

        impl Symbol {
            pub fn new(_pulse0: Pulse, _pulse1: Pulse) -> Self {
                Self::default()
            }
        }

        /// Mock struct for `esp_idf_hal::rmt::Pulse`
        pub struct Pulse;

        impl Pulse {
            pub fn new_with_duration(
                _clock_hz: Hertz,
                _level: super::rmt::PinState,
                _duration: core::time::Duration,
            ) -> Result<Self, crate::mock::esp_idf_sys::EspError> {
                Ok(Self)
            }
        }
    }

    /// Mock module for `esp_idf_hal::units`
    pub mod units {
        pub type ValueType = u32;
        pub type LargeValueType = u64;

        /// Mock struct for `esp_idf_hal::units::Hertz`
        #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
        pub struct Hertz(pub ValueType);
    }
}

/// Mock module for `esp_idf_sys`
pub mod esp_idf_sys {
    use core::fmt;

    /// Mock struct for `esp_idf_sys::EspError`
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct EspError();

    #[cfg(feature = "std")]
    impl std::error::Error for EspError {}

    impl fmt::Display for EspError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt("EspError", f)
        }
    }
}
