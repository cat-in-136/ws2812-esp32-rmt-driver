//! Example demonstrating non-blocking queue/push operations on M5Atom (new RMT API only)
//!
//! This is a practical example showing how to use the non-blocking queue API
//! with the M5Atom (ESP32-PICO-based device with 25 WS2812B LEDs on GPIO27).
//!
//! Compared to `m5atom_smart_leds.rs` (which uses blocking `SmartLedsWrite::write()`),
//! this example uses `Ws2812Esp32RmtDriver` directly with the queue API,
//! allowing non-blocking transmission.

#![cfg(all(feature = "smart-leds-trait", not(feature = "rmt-legacy")))]

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys::*;
use smart_leds::hsv::{hsv2rgb, Hsv};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorGrb24};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

const NUM_LEDS: usize = 25;

fn main() -> ! {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    link_patches();

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio27; // M5Atom built-in LED pin
    let mut driver = Ws2812Esp32RmtDriver::new(led_pin).unwrap();

    // Create a non-blocking transmission queue.
    // The queue must be kept alive (not dropped) while transmission is in progress,
    // otherwise it will block until the transmission completes.
    let mut queue = driver.queue();

    println!("Start non-blocking queue rainbow example on M5Atom!");

    let mut hue = unsafe { esp_random() } as u8;
    loop {
        // Build pixel buffer: rainbow pattern across 25 LEDs
        let mut pixel_data = Vec::with_capacity(NUM_LEDS * 3);
        for i in 0..NUM_LEDS {
            let h = hue.wrapping_add((i * 10) as u8);
            let rgb = hsv2rgb(Hsv {
                hue: h,
                sat: 255,
                val: 32, // low brightness for M5Atom
            });
            let color = LedPixelColorGrb24::new_with_rgb(rgb.r, rgb.g, rgb.b);
            pixel_data.extend_from_slice(color.as_ref());
        }

        // Non-blocking push: returns Err if the previous transmission is still in progress.
        // In that case, fall back to blocking push to avoid dropping frames.
        if queue.push(&pixel_data).is_err() {
            // Previous transmission still in progress; wait for it to complete.
            queue.push_blocking(&pixel_data).unwrap();
        }

        sleep(Duration::from_millis(30));
        hue = hue.wrapping_add(2);
    }
}
