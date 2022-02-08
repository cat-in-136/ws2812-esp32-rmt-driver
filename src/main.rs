use crate::smart_led_lib::Ws2812Rmt;
use esp_idf_sys::*;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds_trait::SmartLedsWrite;
use std::thread::sleep;
use std::time::Duration;

mod smart_led_lib;

fn main() -> ! {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let get_random = || -> u32 { unsafe { esp_random() } };
    let mut ws2812 = Ws2812Rmt::new(0, 27).unwrap();

    loop {
        println!("Hello, world!");

        let rgb = hsv2rgb(Hsv {
            hue: get_random() as u8,
            sat: 255,
            val: 8,
        });
        let pixels = [rgb; 25];
        ws2812.write(pixels.iter().cloned()).unwrap();

        sleep(Duration::from_secs(1));
    }
}
