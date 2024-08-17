# ws2812-esp32-rmt-driver

A rust driver library to control WS2812B (NeoPixel) RGB LED pixels/strips using ESP32 RMT (Remote Control) module.

![Rust](https://github.com/cat-in-136/ws2812-esp32-rmt-driver/workflows/Rust/badge.svg)
[![ws2812-esp32-rmt-driver at crates.io](https://img.shields.io/crates/v/ws2812-esp32-rmt-driver.svg)](https://crates.io/crates/ws2812-esp32-rmt-driver)
[![API](https://docs.rs/ws2812-esp32-rmt-driver/badge.svg)](https://docs.rs/ws2812-esp32-rmt-driver)

By disabling the carrier generator of [the RMT's transmitter][rmt]
, it can be used as just a PWM signal generator for [WS2812B data signal][ws2812b-datasheet]. This control way is the
same as major Arduino/C++ library such as [FastLED](https://github.com/FastLED/FastLED),
[Adafruit_NeoPixel](https://github.com/adafruit/Adafruit_NeoPixel).

The RMT (Remote Control) module is specific to ESP32. Hence, it can be used only for ESP32 SoC.

This library also support SK6812-RGBW 4-color LED pixels/strips (smart-leds API only).

[rmt]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/rmt.html

[ws2812b-datasheet]: https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf

## Usage

Install rust with Xtensa support. Refer [esp-rs/rust-build](https://github.com/esp-rs/rust-build) for the setup
instruction.

Add following dependency to your `Cargo.toml`. Note that version is stripped in this example but it is recommended to
specify version explicitly in your project.

```toml
[dependencies]
esp-idf-sys = { version = "*", features = ["binstart"] }
esp-idf-hal = "*"
smart-leds = "*"

ws2812-esp32-rmt-driver = { version = "*", features = ["smart-leds-trait"] }

[build-dependencies]
embuild = "*"
anyhow = "1"
```

Refer `examples/` directory for the source code.

Make ensure `esp` toolchain is available and `xtensa-esp32-elf-clang` is in your `$PATH`. And then, run as follows

```console
$ cargo build
$ cargo espflash
```

## Features

|Features                |Default|Description                                                           |
|------------------------|-------|----------------------------------------------------------------------|
|`embedded_graphics_core`|       |embedded-graphics API `ws2812_esp32_rmt_driver::lib_embedded_graphics`|
|`smart-leds-trait`      |       |smart-leds API `ws2812_esp32_rmt_driver::lib_smart_leds`              |
|`std`                   |x      |use standard library `std`                                            |
|`alloc`                 |x      |use memory allocator (heap)                                           |

Some examples:

* `features = ["embedded-graphics-core"]` to enable embedded-graphics
  API `ws2812_esp32_rmt_driver::lib_embedded_graphics`.
* `features = ["smart-leds-trait"]` to enable smart-leds API `ws2812_esp32_rmt_driver::lib_smart_leds`.
* default feature to enable just only driver API.

## no_std

To use `no_std`, disable `default` feature. Then, `std` feature is disabled and this library get compatible with `no_std`.

Some examples:

*  `default-feature = false, features = ["alloc", "embedded-graphics-core"]` to enable embedded-graphics API
   `ws2812_esp32_rmt_driver::lib_embedded_graphics` for `no_std` environment with memory allocator.
*  `default-feature = false, features = ["alloc", "smart-leds-trait"]` to enable smart-leds API
   `ws2812_esp32_rmt_driver::lib_smart_leds` for `no_std` environment with memory allocator.
*  `default-feature = false, features = ["embedded-graphics-core"]` to enable embedded-graphics API
   `ws2812_esp32_rmt_driver::lib_embedded_graphics` for `no_std` environment without memory allocator.
*  `default-feature = false, features = ["smart-leds-trait"]` to enable smart-leds API
   `ws2812_esp32_rmt_driver::lib_smart_leds` for `no_std` environment without memory allocator.

When using the memory allocator (heap), enable the `alloc` feature. In this case, most processing works in the same way as `std`.
When not using the memory allocator (heap), leave the `alloc` feature disabled. In this case,
some APIs cannot be used and processing must be changed.
For example, in the embedded-graphics API, the pixel data storage must be prepared by the programmer
using heapless `Vec`-like struct such as `heapless::Vec<u8, X>`.


This library is intended for use with espidf.
For bare-metal environments (i.e. use with [esp-hal](https://crates.io/crates/esp-hal/)),
use the espressif official crate [esp-hal-smartled](https://crates.io/crates/esp-hal-smartled).

## Development

To run the test locally, specify the local toolchain (`stable`, `nightly`, etc...) and target explicitly and disable
example builds (specify `--lib`)
.

```console
$ cargo +stable test --target x86_64-unknown-linux-gnu --lib
```

