use std::path::PathBuf;

use ggez::{Context, GameResult, graphics::{Drawable, Image, Rect}};

#[derive(Debug)]
pub struct Atlas {
    pub path: PathBuf,
    pub image: Image,
    pub tile_size: usize,
    rows: u8,
    cols: u8,
    pub rects: Vec<Rect>,
}

impl Atlas {
    pub fn new(ctx: &Context, path: &str, tile_size: usize, rows: u8, cols: u8) -> GameResult<Self> {
        let path = PathBuf::from(path);
        let image = Image::from_path(ctx, path.as_path())?;
        Ok(Atlas { path, image, tile_size, rows, cols, rects: vec![] })
    }

    pub fn get(&self, id: u32) -> Rect {
        let col = id % (self.cols as u32);
        let row = id / (self.rows as u32);
        let size = self.tile_size as u32;
        self.image.uv_rect(col * size, row * size, size, size)
    }

    pub fn load_rects(&mut self) {
        for i in 0..(self.rows * self.cols) {
            let rect = self.get(i as u32);
            self.rects.push(rect);
        }
    }
}
