//! Mock modules for local testing

/// Mock module for `esp_idf_hal`
pub mod esp_idf_hal {
    pub use super::esp_idf_sys as sys;

    /// Mock module for `esp_idf_hal::gpio`
    pub mod gpio {
        use super::peripheral::Peripheral;
        use paste::paste;

        /// Mock trait for `esp_idf_hal::gpio::OutputPin`.
        pub trait OutputPin {}

        /// Mock struct for `esp_idf_hal::gpio::Pins`.
        #[derive(Debug, Default)]
        pub struct Pins {
            pub gpio0: Gpio0,
            pub gpio1: Gpio1,
            pub gpio2: Gpio2,
            pub gpio3: Gpio3,
            pub gpio4: Gpio4,
            pub gpio5: Gpio5,
            pub gpio6: Gpio6,
            pub gpio7: Gpio7,
            pub gpio8: Gpio8,
            pub gpio9: Gpio9,
            pub gpio10: Gpio10,
            pub gpio11: Gpio11,
            pub gpio12: Gpio12,
            pub gpio13: Gpio13,
            pub gpio14: Gpio14,
            pub gpio15: Gpio15,
            pub gpio16: Gpio16,
            pub gpio17: Gpio17,
            pub gpio18: Gpio18,
            pub gpio19: Gpio19,
            pub gpio20: Gpio20,
            pub gpio21: Gpio21,
        }

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
                        impl Peripheral for [<Gpio $num>] {
                            type P=[<Gpio $num>];
                        }
                    )*
                }
            };
        }
        define_gpio_structs!(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21
        );
    }

    /// Mock module for `esp_idf_hal::peripheral`
    pub mod peripheral {
        /// Mock trait for `esp_idf_hal::peripheral::Peripheral`
        pub trait Peripheral: Sized {
            /// Peripheral singleton type
            type P;
        }
    }

    /// Mock module for `esp_idf_hal::peripherals`
    pub mod peripherals {
        use super::gpio;
        use super::rmt;

        /// Mock struct for `esp_idf_hal::peripherals::Peripherals`
        pub struct Peripherals {
            pub pins: gpio::Pins,
            pub rmt: rmt::RMT,
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
                    rmt: rmt::RMT::new(),
                }
            }
        }
    }

    /// Mock module for `esp_idf_hal::rmt`
    pub mod rmt {
        use super::gpio::OutputPin;
        use super::peripheral::Peripheral;
        use super::sys::EspError;
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

                        impl Peripheral for [<CHANNEL $num>] {
                            type P=[<CHANNEL $num>];
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
            pub fn new<C: RmtChannel>(
                _channel: impl Peripheral<P = C> + 'd,
                _pin: impl Peripheral<P = impl OutputPin> + 'd,
                _config: &TransmitConfig,
            ) -> Result<Self, EspError> {
                Ok(Self { _p: PhantomData })
            }
        }

        /// Mock module for `esp_idf_hal::rmt::config`
        pub mod config {
            /// Mock struct for `esp_idf_hal::rmt::config::TransmitConfig`
            #[derive(Debug, Clone)]
            pub struct TransmitConfig {}

            impl TransmitConfig {
                pub fn new() -> Self {
                    Self {}
                }
                #[allow(unused_mut)]
                pub fn clock_divider(mut self, _divider: u8) -> Self {
                    self
                }
            }
        }
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
