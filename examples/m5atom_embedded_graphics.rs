#![cfg(feature = "embedded-graphics-core")]
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle, Triangle};
use esp_idf_hal::peripherals::Peripherals;
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::lib_embedded_graphics::{LedPixelMatrix, Ws2812DrawTarget};

fn main() -> ! {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio27;
    let channel = peripherals.rmt.channel0;

    let mut draw = Ws2812DrawTarget::<LedPixelMatrix<5, 5>>::new(channel, led_pin).unwrap();
    draw.set_brightness(40);

    println!("Start Ws2812DrawTarget!");

    let mut offset = 0;
    loop {
        draw.clear_with_black().unwrap();

        let shape_width = 5 + 1;
        let shape_offset = |offset: i32, i: usize, count: usize| {
            let mut offset = offset as i32 + shape_width * i as i32;
            if offset < -shape_width {
                offset += shape_width * count as i32;
            }
            offset
        };

        let mut translated_draw = draw.translated(Point::new(shape_offset(offset, 0, 3), 0));
        Circle::new(Point::new(0, 0), 5)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
            .draw(&mut translated_draw)
            .unwrap();

        let mut translated_draw = draw.translated(Point::new(shape_offset(offset, 1, 3), 0));
        Triangle::new(Point::new(0, 0), Point::new(4, 2), Point::new(0, 4))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::GREEN))
            .draw(&mut translated_draw)
            .unwrap();

        let mut translated_draw = draw.translated(Point::new(shape_offset(offset, 2, 3), 0));
        Rectangle::new(Point::new(0, 0), Size::new(5, 5))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
            .draw(&mut translated_draw)
            .unwrap();

        draw.flush().unwrap();

        sleep(Duration::from_millis(100));

        offset -= 1;
        if offset < -shape_width * 3 {
            offset += (-offset / shape_width) * shape_width;
        }
    }
}
