use std::collections::HashMap;

use ggez::{
    Context, GameError, GameResult,
    graphics::{Image, Rect},
};

#[derive(Debug)]
pub struct Atlas {
    pub image: Image,
    pub uv_map: HashMap<String, Rect>,
}

impl Atlas {
    pub fn new(
        ctx: &Context,
        texture_paths: &std::collections::HashSet<String>,
    ) -> GameResult<Self> {
        let mut atlas_width: u32 = 1024;
        let mut atlas_height: u32 = 1024;

        let mut packer = rect_packer::DensePacker::new(atlas_width as i32, atlas_height as i32);
        let mut loaded_images = Vec::new();

        for path in texture_paths {
            let image = match image::open(path) {
                Ok(img) => img,
                Err(_) => {
                    eprintln!("Warning: texture '{path}' not found");
                    image::open(crate::settings::res::MISSING_TEX)
                        .or(Err(GameError::CustomError("No textures found".to_owned())))?
                }
            }
            .to_rgba8();

            let (w, h) = image.dimensions();
            let rect;
            loop {
                match packer.pack(w as i32, h as i32, false) {
                    Some(r) => {
                        rect = r;
                        break;
                    }

                    None => {
                        atlas_width *= 2;
                        atlas_height *= 2;

                        if atlas_width > 8192 || atlas_height > 8192 {
                            return Err(GameError::CustomError("Atlas full".to_owned()));
                        }

                        packer.resize(atlas_width as i32, atlas_height as i32);
                    }
                }
            }

            loaded_images.push((rect, image, path.clone()));
        }

        let mut atlas_buffer = image::RgbaImage::new(atlas_width, atlas_height);

        for (rect, img, _) in loaded_images.clone() {
            for x in 0..rect.width as u32 {
                for y in 0..rect.height as u32 {
                    atlas_buffer.put_pixel(
                        u32::try_from(rect.x).expect("u32 can't be negative") + x,
                        u32::try_from(rect.y).expect("u32 can't be negative") + y,
                        *img.get_pixel(x, y),
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

    pub fn make_texture_rect(&self, texture_path: &str) -> GameResult<crate::ecs::UV> {
        Ok(crate::ecs::UV(*self.uv_map.get(texture_path).ok_or(
            GameError::ResourceNotFound(
                format!("Texture {texture_path} not found in atlas"),
                vec![],
            ),
        )?))
    }

    pub fn get_block_uv(&self, block_def: &crate::defs::BlockDef) -> GameResult<&Rect> {
        self.uv_map
            .get(&block_def.texture)
            .ok_or(GameError::ResourceNotFound(
                format!("Texture {} not found in atlas", block_def.texture),
                vec![],
            ))
    }

    pub fn get_item_uv(&self, item_def: &crate::defs::ItemDef) -> GameResult<&Rect> {
        self.uv_map
            .get(&item_def.texture)
            .ok_or(GameError::ResourceNotFound(
                format!("Texture {} not found in atlas", item_def.texture),
                vec![],
            ))
    }

    pub fn get_ui_uv<T: crate::ui::UI>(&self) -> GameResult<&Rect> {
        let path = T::get_texture_path();
        self.uv_map.get(path).ok_or(GameError::ResourceNotFound(
            format!("Texture {path} not found in atlas"),
            vec![],
        ))
    }
}
