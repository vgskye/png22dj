mod utils;

use image::{imageops::FilterType, DynamicImage};
use imagequant::{Attributes, Image, RGBA};
use rgb::FromSlice;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
struct ProcessedImage {
    palette: Vec<RGBA>,
    bitmap: Vec<u8>,
    #[wasm_bindgen(readonly)]
    pub width: u32,
    #[wasm_bindgen(readonly)]
    pub height: u32,
}

#[derive(Serialize)]
struct Serialized2dj<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tooltip: Option<String>,
    palette: &'a [u32],
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct Serialized2dja<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    width: u32,
    height: u32,
    pages: Vec<Serialized2dj<'a>>,
}

#[wasm_bindgen]
impl ProcessedImage {
    pub fn draw(&self, data: &mut [u8]) {
        let buf = data.as_rgba_mut();
        self.bitmap
            .iter()
            .map(|&idx| self.palette[idx as usize])
            .map(|color| color.map_alpha(|a| if a == 0 { 0 } else { 255 }))
            .zip(buf.iter_mut())
            .for_each(|(pixel, slot)| *slot = pixel);
        
    }

    pub fn serialize(&self, name: Option<String>) -> Vec<u8> {
        let width = self.width.div_ceil(128);
        let height = self.height.div_ceil(128);
        let x_off = ((width * 128) - self.width) / 2;
        let y_off = ((height * 128) - self.height) / 2;

        let transparent_idx = (self.palette.len() - 1) as u8;
        let palette: Vec<_> = self
            .palette
            .iter()
            .take(transparent_idx as usize)
            .map(|color| ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32))
            .collect();

        let tiles: Vec<_> = coords(width, height)
            .map(|(tile_x, tile_y)| {
                let pixels: Vec<_> = coords(128, 128)
                    .map(move |(inner_x, inner_y)| {
                        let output_x_range = x_off..(x_off + self.width);
                        let output_y_range = y_off..(y_off + self.height);
                        let output_x = (tile_x * 128) + inner_x;
                        let output_y = (tile_y * 128) + inner_y;
                        if output_x_range.contains(&output_x) && output_y_range.contains(&output_y)
                        {
                            let input_x = output_x - x_off;
                            let input_y = output_y - y_off;
                            let idx = (input_y * self.width) + input_x;
                            let quant_color = self.bitmap[idx as usize];
                            if quant_color == transparent_idx {
                                0
                            } else {
                                quant_color + 1
                            }
                        } else {
                            0
                        }
                    })
                    .collect();
                Serialized2dj {
                    label: name.clone(),
                    tooltip: if width != 1 || height != 1 {
                        Some(format!("X: {tile_x}, Y: {tile_y}"))
                    } else {
                        None
                    },
                    palette: &palette,
                    pixels,
                    width: 128,
                    height: 128,
                }
            })
            .collect();
        let result = Serialized2dja {
            title: name,
            width,
            height,
            pages: tiles,
        };
        serde_json::to_string(&result).unwrap().into_bytes()
    }
}

fn coords(w: u32, h: u32) -> impl Iterator<Item = (u32, u32)> {
    (0..h).flat_map(move |y| (0..w).map(move |x| (x, y)))
}

#[wasm_bindgen(js_name = Context)]
struct WrappedAttributes(Attributes);

#[wasm_bindgen(js_class = Context)]
impl WrappedAttributes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WrappedAttributes, String> {
        let mut attrs = Attributes::new();
        attrs.set_max_colors(64).unwrap();
        attrs.set_last_index_transparent(true);
        Ok(Self(attrs))
    }

    pub fn process(&self, image: &WrappedImage, w: u32, h: u32) -> Result<ProcessedImage, String> {
        let image_buf = image
            .0
            .resize(w * 128, h * 128, FilterType::Lanczos3)
            .to_rgba8();
        let rgba_buf = image_buf.as_raw().as_rgba();
        let mut quant_img = Image::new_borrowed(
            &self.0,
            rgba_buf,
            image_buf.width() as usize,
            image_buf.height() as usize,
            0.0,
        )
        .map_err(|e| e.to_string())?;
        quant_img
            .add_fixed_color(RGBA::new(0, 0, 0, 0))
            .map_err(|e| e.to_string())?;
        let mut quantized = self.0.quantize(&mut quant_img).map_err(|e| e.to_string())?;
        quantized
            .set_dithering_level(1.0)
            .map_err(|e| e.to_string())?;
        let (palette, bitmap) = quantized
            .remapped(&mut quant_img)
            .map_err(|e| e.to_string())?;
        Ok(ProcessedImage {
            palette,
            bitmap,
            width: image_buf.width(),
            height: image_buf.height(),
        })
    }
}

#[wasm_bindgen(js_name = Image)]
struct WrappedImage(DynamicImage);

#[wasm_bindgen(js_class = Image)]
impl WrappedImage {
    #[wasm_bindgen(constructor)]
    pub fn new(buf: &[u8]) -> Result<WrappedImage, String> {
        image::load_from_memory(buf)
            .map(WrappedImage)
            .map_err(|e| e.to_string())
    }
}
