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
|`rmt-legacy`            |      |Use legacy RMT API (backward compatible, no_std + no_alloc support). Enables `esp-idf-hal/rmt-legacy`|

Some examples:

* `features = ["embedded-graphics-core"]` to enable embedded-graphics
  API `ws2812_esp32_rmt_driver::lib_embedded_graphics`.
* `features = ["smart-leds-trait"]` to enable smart-leds API `ws2812_esp32_rmt_driver::lib_smart_leds`.
* default feature to enable just only driver API.

## RMT API modes

This library supports two RMT API modes:
* **`rmt-legacy`** (optional, not default): Uses the legacy `TxRmtDriver` API from esp-idf-hal. Enabling this feature also enables `esp-idf-hal/rmt-legacy`. This is fully backward compatible with existing code. It supports `no_std` without allocator (via `heapless::Vec`).

* **New API** (default, when `rmt-legacy` is NOT enabled): Uses the new `TxChannelDriver` API from esp-idf-hal 0.46+. This removes the need to manually specify an RMT channel. The `alloc` feature is required.

When `rmt-legacy` feature is not enabled, the new API is automatically selected.

### Legacy RMT API (rmt-legacy) vs New API

```toml
// new API - rmt-legacy not enabled
[dependencies]
ws2812-esp32-rmt-driver = { features = ["smart-leds-trait"] }

// rmt-legacy
[dependencies]
ws2812-esp32-rmt-driver = { features = ["rmt-legacy", "smart-leds-trait"] }
```

Code changes:
```rust
// new API - rmt-legacy not enabled
let mut driver = Ws2812Esp32Rmt::new(led_pin)?;

// rmt-legacy
let channel = peripherals.rmt.channel0;
let mut driver = Ws2812Esp32Rmt::new(channel, led_pin)?;
```

For custom config:
```rust
// new API - rmt-legacy not enabled
let config = TxChannelConfig {
    resolution: Hertz(10_000_000),
    ..Default::default()
};
let driver = Ws2812Esp32RmtDriver::new_with_config(led_pin, &config)?;

// rmt-legacy
let config = TransmitConfig::new().clock_divider(1);
let tx = TxRmtDriver::new(channel, led_pin, &config)?;
let driver = Ws2812Esp32RmtDriver::new_with_rmt_driver(tx)?;
```

The new API also provides non-blocking queue operations:
```rust
let mut driver = Ws2812Esp32RmtDriver::new(led_pin)?;
let mut queue = driver.queue();
queue.push(&pixel_data)?;  // non-blocking
queue.push_blocking(&pixel_data)?;  // blocking
```

## no_std

To use `no_std`, disable `default` feature. Then, `std` feature is disabled and this library get compatible with `no_std`.

Some examples:

*  `default-feature = false, features = ["alloc", "rmt-legacy", "embedded-graphics-core"]` to enable embedded-graphics API
   `ws2812_esp32_rmt_driver::lib_embedded_graphics` for `no_std` environment with memory allocator.
*  `default-feature = false, features = ["alloc", "rmt-legacy", "smart-leds-trait"]` to enable smart-leds API
   `ws2812_esp32_rmt_driver::lib_smart_leds` for `no_std` environment with memory allocator.
*  `default-feature = false, features = ["rmt-legacy", "embedded-graphics-core"]` to enable embedded-graphics API
   `ws2812_esp32_rmt_driver::lib_embedded_graphics` for `no_std` environment without memory allocator.
*  `default-feature = false, features = ["rmt-legacy", "smart-leds-trait"]` to enable smart-leds API
   `ws2812_esp32_rmt_driver::lib_smart_leds` for `no_std` environment without memory allocator.

When using the memory allocator (heap), enable the `alloc` feature. In this case, most processing works in the same way as `std`.
When not using the memory allocator (heap), leave the `alloc` feature disabled. In this case,
some APIs cannot be used and processing must be changed.
For example, in the embedded-graphics API, the pixel data storage must be prepared by the programmer
using heapless `Vec`-like struct such as `heapless::Vec<u8, X>`.

Note: The new RMT API mode (when `rmt-legacy` feature is NOT enabled) requires the `alloc` feature and does not support `no_std` without allocator.


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

