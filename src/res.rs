use std::path::PathBuf;

use ggez::{Context, GameResult, graphics::{Image, Rect}};

use crate::world::TileType;

#[derive(Debug)]
pub struct Atlas {
    pub path: PathBuf,
    pub image: Image,
    pub tile_size: usize,
    rows: u8,
    cols: u8,
}

impl Atlas {
    pub fn new(ctx: &Context, path: &str, tile_size: usize, rows: u8, cols: u8) -> GameResult<Self> {
        let path = PathBuf::from(path);
        let image = Image::from_path(ctx, path.as_path())?;
        Ok(Atlas { path, image, tile_size, rows, cols })
    }

    pub fn get(&self, tile_id: &TileType) -> Option<Rect> {
        let id = *tile_id as u32;
        if id >= (self.rows as u32) * (self.cols as u32) {
            return None
        } else {
            let col = id % (self.cols as u32);
            let row = id / (self.rows as u32);
            let size = self.tile_size as u32;
            Some(self.image.uv_rect(col * size, row * size, size, size))
        }
    }
}
