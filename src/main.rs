use esp_idf_sys::*;
use std::cmp::min;
use std::ops::Range;
use std::os::raw::*;
use std::thread::sleep;
use std::time::Duration;

const WS2812_TO0H_NS: u16 = 400;
const WS2812_TO0L_NS: u16 = 850;
const WS2812_TO1H_NS: u16 = 800;
const WS2812_TO1L_NS: u16 = 450;

unsafe extern "C" fn ws2821_rmt_adapter(
    src: *const c_void,
    dest: *mut rmt_item32_s,
    src_size: u32,
    wanted_num: u32,
    translated_size: *mut u32,
    item_num: *mut u32,
) {
    if src.is_null() || dest.is_null() {
        *translated_size = 0;
        *item_num = 0;
        return;
    }

    let ws2812_channel = rmt_channel_t_RMT_CHANNEL_0; // TODO

    let mut clock_hz = 0u32;
    esp!({ rmt_get_counter_clock(ws2812_channel, &mut clock_hz as *mut u32) });
    let clock_hz = clock_hz as u64;
    let to0h_clk = ((WS2812_TO0H_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
    let to0l_clk = ((WS2812_TO0L_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
    let to1h_clk = ((WS2812_TO1H_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
    let to1l_clk = ((WS2812_TO1L_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
    let bit0_val = to0h_clk | (1 << 15) | (to0l_clk << 16) | (0 << 31);
    let bit1_val = to1h_clk | (1 << 15) | (to1l_clk << 16) | (0 << 31);

    let src_len = min(src_size, wanted_num / 8) as usize;
    let src_slice = std::slice::from_raw_parts(src as *const u8, src_len);
    let mut dest_slice = std::slice::from_raw_parts_mut(dest, src_slice.len() * 8);

    src_slice
        .iter()
        .flat_map(|v| {
            (0..8).map(move |i| {
                if v & (1 << (7 - i)) != 0 {
                    bit1_val
                } else {
                    bit0_val
                }
            })
        })
        .enumerate()
        .for_each(|(i, v)| {
            dest_slice[i].__bindgen_anon_1.val = v;
        });

    *translated_size = src_slice.len() as _;
    *item_num = dest_slice.len() as _;
}

fn ws2812_setup() -> Result<(), EspError> {
    let ws2812_channel = rmt_channel_t_RMT_CHANNEL_0;
    let ws2812_pin = 27;

    let rmt_cfg = rmt_config_t {
        rmt_mode: rmt_mode_t_RMT_MODE_TX,
        channel: ws2812_channel,
        gpio_num: ws2812_pin,
        clk_div: 2,
        mem_block_num: 1,
        __bindgen_anon_1: rmt_config_t__bindgen_ty_1 {
            tx_config: rmt_tx_config_t {
                loop_en: false,
                carrier_level: rmt_carrier_level_t_RMT_CARRIER_LEVEL_HIGH,
                carrier_en: false,
                idle_level: rmt_idle_level_t_RMT_IDLE_LEVEL_LOW,
                idle_output_en: true,
                ..Default::default()
            },
        },
        ..Default::default()
    };
    esp!(unsafe { rmt_config(&rmt_cfg) })?;
    esp!(unsafe { rmt_driver_install(ws2812_channel, 0, 0) })?;

    esp!(unsafe { rmt_translator_init(ws2812_channel, Some(ws2821_rmt_adapter)) })?;

    Ok(())
}

fn ws2812_update() -> Result<(), EspError> {
    let ws2812_channel = rmt_channel_t_RMT_CHANNEL_0;

    let get_random = || -> u32 { unsafe { esp_random() } };

    const led_num: usize = 5 * 5;
    const buffer_size: usize = led_num * 3;
    let mut buffer = [0u8; buffer_size];

    {
        let grb = (
            (get_random() as u8) & 0x0F,
            (get_random() as u8) & 0x0F,
            (get_random() as u8) & 0x0F,
        );
        for i in 0..(buffer_size / 3) {
            buffer[i * 3 + 0] = grb.0;
            buffer[i * 3 + 1] = grb.1;
            buffer[i * 3 + 2] = grb.2;
        }
    }

    esp!(unsafe { rmt_write_sample(ws2812_channel, buffer.as_ptr(), buffer_size as u32, false) })?;
    esp!(unsafe { rmt_wait_tx_done(ws2812_channel, 100) })?;

    Ok(())
}

fn main() -> ! {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    ws2812_setup().unwrap();

    loop {
        println!("Hello, world!");
        ws2812_update().unwrap();
        sleep(Duration::from_secs(1));
    }
}
