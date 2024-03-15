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

* `features = ["embedded-graphics-core"]` to enable embedded-graphics
  API `ws2812_esp32_rmt_driver::lib_embedded_graphics`.
* `features = ["smart-leds-trait"]` to enable smart-leds API `ws2812_esp32_rmt_driver::lib_smart_leds`.
* default feature to enable just only driver API.

## Development

To run the test locally, specify the local toolchain (`stable`, `nightly`, etc...) and target explicitly and disable
example builds (specify `--lib`)
.

```console
$ cargo +stable test --target x86_64-unknown-linux-gnu --lib
```

