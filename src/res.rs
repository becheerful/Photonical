use std::collections::HashMap;

use ggez::{
    Context, GameResult,
    graphics::{Image, Rect},
};

#[derive(Debug)]
pub struct Atlas {
    pub image: Image,
    pub uv_map: HashMap<String, Rect>,
}

impl Atlas {
    pub fn new(ctx: &Context, texture_paths: &[String]) -> GameResult<Self> {
        let mut packer = rect_packer::DensePacker::new(4096, 4096);
        let mut loaded_images = Vec::new();

        for path in texture_paths {
            let image = match image::open(path) {
                Ok(img) => img,
                Err(_) => {
                    eprintln!("Warning: texture '{path}' not found");
                    image::open(crate::MISSING_TEX).or(Err(ggez::GameError::CustomError(
                        "No textures found".to_owned(),
                    )))?
                }
            }
            .to_rgba8();
            let (w, h) = image.dimensions();
            let rect = packer
                .pack(
                    w.try_into().expect("The value is outside the range of i32"),
                    h.try_into().expect("The value is outside the range of i32"),
                    false,
                )
                .ok_or(ggez::GameError::CustomError("Atlas full".to_owned()))?;
            loaded_images.push((rect, image, path.clone()));
        }

        let atlas_width = u32::try_from(packer.size().0).expect("u32 can't be negative");
        let atlas_height = u32::try_from(packer.size().1).expect("u32 can't be negative");

        let mut atlas_buffer = image::RgbaImage::new(atlas_width, atlas_height);

        for (rect, img, _) in loaded_images.clone() {
            for y in 0..u32::try_from(rect.height).expect("u32 can't be negative") {
                for x in 0..u32::try_from(rect.width).expect("u32 can't be negative") {
                    let px = img.get_pixel(x, y);
                    atlas_buffer.put_pixel(
                        u32::try_from(rect.x).expect("u32 can't be negative") + x,
                        u32::try_from(rect.y).expect("u32 can't be negative") + y,
                        *px,
                    );
                }
            }
        }

        let raw = atlas_buffer.into_raw();
        let image = Image::from_pixels(
            ctx,
            &raw,
            ggez::graphics::ImageFormat::Rgba8UnormSrgb,
            atlas_width,
            atlas_height,
        );

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

        Ok(Self { image, uv_map })
    }

    pub fn get_block_uv(&self, block_def: &crate::defs::BlockDef) -> GameResult<&Rect> {
        self.uv_map
            .get(&block_def.texture)
            .ok_or(ggez::GameError::ResourceNotFound(
                "Texture not found in atlas".to_owned(),
                vec![],
            ))
    }

    pub fn get_item_uv(&self, item_def: &crate::defs::ItemDef) -> GameResult<&Rect> {
        self.uv_map
            .get(&item_def.texture)
            .ok_or(ggez::GameError::ResourceNotFound(
                "Texture not found in atlas".to_owned(),
                vec![],
            ))
    }

    pub fn get_ui_uv(&self, uv: &impl crate::ui::UI) -> GameResult<&Rect> {
        self.uv_map
            .get(uv.get_texture_path())
            .ok_or(ggez::GameError::ResourceNotFound(
                "Texture not found in atlas".to_owned(),
                vec![],
            ))
    }
}
