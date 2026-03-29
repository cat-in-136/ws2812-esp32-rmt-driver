use esp_idf_hal::peripherals::Peripherals;
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColor, LedPixelColorRgb24};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

const NUM_LEDS: usize = 1;

fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Initializing...");

    let peripherals = Peripherals::take().unwrap();
    let led_pin = peripherals.pins.gpio8;
    let mut driver = Ws2812Esp32RmtDriver::new(led_pin).unwrap();
    // Create a non-blocking transmission queue, be careful that it _will_ block if dropped.
    let mut queue = driver.queue();

    log::info!("Start (non-)blocking queue/push example!");

    let mut brightness: u8 = 0;
    loop {
        // Build pixel buffer: pulsing red.
        let color = LedPixelColorRgb24::new_with_rgb(brightness, 0, 0);
        let pixel_bytes: [u8; 3] = color.as_ref().try_into().unwrap();
        let pixel_data: Vec<u8> = pixel_bytes
            .iter()
            .copied()
            .cycle()
            .take(NUM_LEDS * 3)
            .collect();

        // This is more test code than example code. This demonstrates that the
        // non-blocking write will fail if the previous transmission is still in progress.
        log::info!("Calling push(), brightness={brightness}");
        match queue.push(&pixel_data) {
            Ok(()) => {
                let mut i = 0;
                // Since we do not block, this will fail a few times.
                while queue.push(&pixel_data).is_err() {
                    i += 1;
                }
                log::info!("Transmission failed {i} times after initial success.");
            }
            Err(e) => log::error!("write() error: {e}"),
        }

        // However, push_blocking... blocks.
        log::info!("Calling push_blocking(), brightness={brightness}");
        match queue.push_blocking(&pixel_data) {
            Ok(()) => {
                let mut i = 0;
                // Since we block, this must immediately succeed.
                while queue.push_blocking(&pixel_data).is_err() {
                    i += 1;
                }
                if i == 0 {
                    log::info!(
                        "Blocking: Transmission succeeded immediately after initial success."
                    );
                } else {
                    log::error!("Blocking: Transmission failed {i} times after initial success, that should not happen!");
                }
            }
            Err(e) => log::error!("write() error: {e}"),
        }

        // This is more of a real example, push and do something else.
        log::info!("Calling write(), brightness={brightness}");
        match queue.push(&pixel_data) {
            Ok(()) => {
                log::info!("Transmission started, doing other work...");
            }
            Err(e) => log::error!("write() error: {e}"),
        }

        sleep(Duration::from_millis(100));
        brightness = brightness.wrapping_add(20);
    }
}
