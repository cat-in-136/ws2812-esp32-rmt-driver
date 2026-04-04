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
        use super::gpio;

        /// Mock struct for `esp_idf_hal::peripherals::Peripherals`
        pub struct Peripherals {
            pub pins: gpio::Pins,
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
                    pins: gpio::Pins::new(),
                }
            }
        }
    }

    /// Mock module for `esp_idf_hal::rmt`
    pub mod rmt {
        use super::gpio::OutputPin;
        use super::sys::EspError;
        use config::TxChannelConfig;
        use core::marker::PhantomData;

        /// Mock struct for `esp_idf_hal::rmt::TxChannelDriver`
        pub struct TxChannelDriver<'d> {
            _p: PhantomData<&'d mut ()>,
        }

        impl<'d> TxChannelDriver<'d> {
            /// Initialize the mock of `TxChannelDriver`.
            /// No argument is used in this mock.
            pub fn new(
                _pin: impl OutputPin + 'd,
                _config: &TxChannelConfig,
            ) -> Result<Self, EspError> {
                Ok(Self { _p: PhantomData })
            }
        }

        /// Mock module for `esp_idf_hal::rmt::encoder`
        pub mod encoder {
            pub struct BytesEncoder {}
        }

        /// Mock struct for `esp_idf_hal::rmt::config`
        pub mod config {
            use super::super::units::Hertz;

            /// Mock struct for `esp_idf_hal::rmt::config::TxChannelConfig`
            #[derive(Debug, Clone)]
            pub struct TxChannelConfig {
                pub resolution: Hertz,
            }

            impl Default for TxChannelConfig {
                fn default() -> Self {
                    Self {
                        resolution: Hertz(10_000_000),
                    }
                }
            }

            /// Mock struct for `esp_idf_hal::rmt::config::TransmitConfig`
            #[derive(Debug, Clone, Default)]
            pub struct TransmitConfig {}
        }
    }

    /// Mock module for `esp_idf_hal::units`
    pub mod units {
        pub type ValueType = u32;
        pub type LargeValueType = u64;

        /// Mock struct for `esp_idf_hal::units::Hertz`
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default, Debug)]
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
