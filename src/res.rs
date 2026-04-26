use std::collections::HashMap;

use ggez::{Context, GameError, GameResult, graphics::{Image, ImageFormat, Rect}};
use image::RgbaImage;
use rect_packer::DensePacker;

use crate::defs::{BlockDef, ItemDef};

#[derive(Debug)]
pub struct Atlas {
    pub image: Image,
    pub uv_map: HashMap<String, Rect>,
}

impl Atlas {
    pub fn new(ctx: &Context, texture_paths: &[String]) -> GameResult<Self> {
        let (image, uv_map) = Self::pack_textures(ctx, texture_paths)?;
        Ok(Atlas { image, uv_map })
    }

    fn pack_textures(ctx: &Context, paths: &[String]) -> GameResult<(Image, HashMap<String, Rect>)> {
        let mut packer = DensePacker::new(2048, 2048);
        let mut loaded_images = Vec::new();

        for path in paths {
            let image = match image::open(path) {
                Ok(img) => img,
                Err(_) => {
                    eprintln!("Warning: texture '{}' not found", path);
                    image::open("./resources/missing.png").unwrap()
                }
            }.to_rgba8();
            let (w, h) = image.dimensions();
            let rect = packer.pack(w as i32, h as i32, false).ok_or_else(|| GameError::CustomError("Atlas full".into()))?;
            loaded_images.push((rect, image, path.clone()));
        }
        
        let atlas_width = packer.size().0 as u32;
        let atlas_height = packer.size().1 as u32;
        let mut atlas_buffer = RgbaImage::new(atlas_width, atlas_height);

        for (rect, img, _path) in loaded_images.clone() {
            for y in 0..rect.height {
                for x in 0..rect.width {
                    let px = img.get_pixel(x as u32, y as u32);
                    atlas_buffer.put_pixel((rect.x + x) as u32, (rect.y + y) as u32, *px);
                }
            }
        }

        let raw = atlas_buffer.into_raw();
        let ggez_image = Image::from_pixels(ctx, &raw, ImageFormat::Rgba8UnormSrgb, atlas_width, atlas_height);

        let mut uv_map = HashMap::new();
        for (rect, _img, path) in loaded_images.clone() {
            let uv = Rect::new(
                rect.x as f32 / atlas_width as f32,
                rect.y as f32 / atlas_height as f32,
                rect.width as f32 / atlas_width as f32,
                rect.height as f32 / atlas_height as f32,  
            );
            uv_map.insert(path, uv);
        }

        Ok((ggez_image, uv_map))
    }

    pub fn get_block_uv(&self, block_def: &BlockDef) -> &Rect {
        self.uv_map.get(&block_def.texture).expect("Texture not found in atlas")
    }

    pub fn get_item_uv(&self, item_def: &ItemDef) -> &Rect {
        self.uv_map.get(&item_def.texture).expect("Texture not found in atlas")
    }
}
