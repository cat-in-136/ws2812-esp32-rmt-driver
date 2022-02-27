use crate::driver::color::LedPixelColorGrb24;
use esp_idf_sys::*;
use once_cell::sync::OnceCell;
use std::cmp::min;
use std::ffi::c_void;

const WS2812_TO0H_NS: u16 = 400;
const WS2812_TO0L_NS: u16 = 850;
const WS2812_TO1H_NS: u16 = 800;
const WS2812_TO1L_NS: u16 = 450;

static WS2821_ITEM_ENCODER: OnceCell<Ws2812Esp32RmtItemEncoder> = OnceCell::new();

#[repr(C)]
struct Ws2812Esp32RmtItemEncoder {
    bit0: u32,
    bit1: u32,
}

impl Ws2812Esp32RmtItemEncoder {
    fn new(channel: rmt_channel_t) -> Result<Self, EspError> {
        let mut clock_hz = 0u32;
        esp!(unsafe { rmt_get_counter_clock(channel, &mut clock_hz as *mut u32) })?;
        let clock_hz = clock_hz as u64;
        let to0h_clk = ((WS2812_TO0H_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
        let to0l_clk = ((WS2812_TO0L_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
        let to1h_clk = ((WS2812_TO1H_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
        let to1l_clk = ((WS2812_TO1L_NS as u64) * clock_hz.clone() / 1000_000_000) as u32;
        let bit0 = to0h_clk | (1 << 15) | (to0l_clk << 16) | (0 << 31);
        let bit1 = to1h_clk | (1 << 15) | (to1l_clk << 16) | (0 << 31);
        Ok(Self { bit0, bit1 })
    }

    fn encode(&self, src_slice: &[u8], dest_slice: &mut [rmt_item32_s]) {
        src_slice
            .iter()
            .flat_map(|v| {
                (0..8).map(move |i| {
                    if v & (1 << (7 - i)) != 0 {
                        self.bit1
                    } else {
                        self.bit0
                    }
                })
            })
            .enumerate()
            .for_each(|(i, v)| {
                dest_slice[i].__bindgen_anon_1.val = v;
            });
    }
}

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

    let src_len = min(src_size, wanted_num / 8) as usize;
    let src_slice = std::slice::from_raw_parts(src as *const u8, src_len);
    let dest_slice = std::slice::from_raw_parts_mut(dest, src_slice.len() * 8);

    if let Some(encoder) = WS2821_ITEM_ENCODER.get() {
        encoder.encode(src_slice, dest_slice)
    }

    *translated_size = src_slice.len() as _;
    *item_num = dest_slice.len() as _;
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Ws2812Esp32RmtDriverError(#[from] EspError);

pub struct Ws2812Esp32RmtDriver {
    channel: rmt_channel_t,
    pub wait_tx_done: bool,
}

impl Ws2812Esp32RmtDriver {
    pub fn new(channel_num: u8, gpio_num: u32) -> Result<Self, Ws2812Esp32RmtDriverError> {
        let channel = channel_num as rmt_channel_t;
        let gpio_num = gpio_num as gpio_num_t;
        let clk_div = 2;

        let rmt_cfg = rmt_config_t {
            rmt_mode: rmt_mode_t_RMT_MODE_TX,
            channel,
            gpio_num,
            clk_div,
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
        esp!(unsafe { rmt_driver_install(channel, 0, 0) })?;
        esp!(unsafe { rmt_translator_init(channel, Some(ws2821_rmt_adapter)) })?;

        let _encoder =
            WS2821_ITEM_ENCODER.get_or_try_init(|| Ws2812Esp32RmtItemEncoder::new(channel))?;

        Ok(Self {
            channel,
            wait_tx_done: true,
        })
    }

    pub fn write(&mut self, grb_pixels: &[u8]) -> Result<(), Ws2812Esp32RmtDriverError> {
        esp!(unsafe {
            let grb_ptr = grb_pixels.as_ptr();
            rmt_write_sample(
                self.channel,
                grb_ptr,
                grb_pixels.len() as u32,
                self.wait_tx_done,
            )
        })?;
        Ok(())
    }

    pub fn write_colors<I>(&mut self, iterator: I) -> Result<(), Ws2812Esp32RmtDriverError>
    where
        I: IntoIterator<Item = LedPixelColorGrb24>,
    {
        let mut vec = Vec::new();
        for color in iterator {
            for v in color.as_ref() {
                vec.push(*v);
            }
        }
        self.write(&vec)
    }
}

impl Drop for Ws2812Esp32RmtDriver {
    fn drop(&mut self) {
        esp!(unsafe { rmt_driver_uninstall(self.channel) }).unwrap()
    }
}
