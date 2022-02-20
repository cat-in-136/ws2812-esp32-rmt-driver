# ws2812-esp32-rmt-driver

A rust driver library to control WS2812B (NeoPixel) RGB LED pixels/strips using ESP32 RMT (Remote Control) module.

By disabling the carrier generator of [the RMT's transmitter][rmt]
, it can be used as just a PWM signal generator for [WS2812B data signal][ws2812b-datasheet]. This control way is the
same as major Arduino/C++ library such as [FastLED](https://github.com/FastLED/FastLED),
[Adafruit_NeoPixel](https://github.com/adafruit/Adafruit_NeoPixel).

The RMT (Remote Control) module is specific to ESP32. Hence, it can be used only for ESP32 SoC.

[rmt]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/rmt.html

[ws2812b-datasheet]: https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf

## Usage

Install rust with Xtensa support. Refer [rpm-rs/rust-build](https://github.com/esp-rs/rust-build) for the setup
instruction.

Add following dependency to your `Cargo.toml`. Note that version is stripped in this example but it is recommended to
specify version explicitly in your project.

```toml
[dependencies]
esp-idf-sys = { version = "*", features = ["binstart"] }
smart-leds = "*"

ws2812-esp32-rmt-driver = "*"

[build-dependencies]
embuild = "*"
anyhow = "1"
```

Refer `examples/` directory for the source code.

Make ensure `esp` toolchain is available and `xtensa-esp32-elf-clang` is in your `$PATH`. And then, run as follows

```console
$ cargo build
$ cargo espflush
```
