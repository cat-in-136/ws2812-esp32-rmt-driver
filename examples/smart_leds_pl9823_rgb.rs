#![cfg(feature = "smart-leds-trait")]
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::esp_random;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds_trait::{SmartLedsWrite, RGB8};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorRgb24;
use ws2812_esp32_rmt_driver::LedPixelEsp32Rmt;

fn main() -> ! {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    // Shenzhen Rita Lighting PL9823
    const PL9823_T0H_NS: Duration = Duration::from_nanos(350);
    const PL9823_T0L_NS: Duration = Duration::from_nanos(1360);
    const PL9823_T1H_NS: Duration = Duration::from_nanos(1360);
    const PL9823_T1L_NS: Duration = Duration::from_nanos(350);

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio25;

    #[cfg(not(feature = "rmt-legacy"))]
    let mut ws2812 = {
        use esp_idf_hal::rmt::config::TxChannelConfig;
        use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriverBuilder;

        let config = TxChannelConfig {
            resolution: esp_idf_hal::units::Hertz(10_000_000),
            ..Default::default()
        };
        let ws2812_driver = Ws2812Esp32RmtDriverBuilder::new_with_config(led_pin, &config)
            .unwrap()
            .encoder_duration(
                &PL9823_T0H_NS,
                &PL9823_T0L_NS,
                &PL9823_T1H_NS,
                &PL9823_T1L_NS,
            )
            .unwrap()
            .build()
            .unwrap();
        LedPixelEsp32Rmt::<RGB8, LedPixelColorRgb24>::new_with_ws2812_driver(ws2812_driver).unwrap()
    };

    #[cfg(feature = "rmt-legacy")]
    let mut ws2812 = {
        use esp_idf_hal::rmt::config::TransmitConfig;
        use esp_idf_hal::rmt::TxRmtDriver;
        use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriverBuilder;

        let channel = peripherals.rmt.channel0;
        let driver_config = TransmitConfig::new().clock_divider(1); // Required parameter.
        let tx_driver = TxRmtDriver::new(channel, led_pin, &driver_config).unwrap();
        let ws2812_driver = Ws2812Esp32RmtDriverBuilder::new_with_rmt_driver(tx_driver)
            .unwrap()
            .encoder_duration(
                &PL9823_T0H_NS,
                &PL9823_T0L_NS,
                &PL9823_T1H_NS,
                &PL9823_T1L_NS,
            )
            .unwrap()
            .build()
            .unwrap();
        LedPixelEsp32Rmt::<RGB8, LedPixelColorRgb24>::new_with_ws2812_driver(ws2812_driver).unwrap()
    };

    println!("Start NeoPixel rainbow!");

    let mut hue = unsafe { esp_random() } as u8;
    loop {
        let pixels = std::iter::repeat(hsv2rgb(Hsv {
            hue,
            sat: 255,
            val: 8,
        }))
        .take(25);
        ws2812.write(pixels).unwrap();

        sleep(Duration::from_millis(100));

        hue = hue.wrapping_add(10);
    }
}
