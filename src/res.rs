use std::path::PathBuf;

use ggez::{Context, GameResult, glam::Vec2, graphics::{Image, Rect}};

use crate::world::World;

#[derive(Debug)]
pub struct Atlas {
    pub image: Image,
    pub aspect: Vec2,
    tile_size: usize,
    rows: u8,
    cols: u8,
    pub rects: Vec<Rect>,
}

impl Atlas {
    pub fn new(ctx: &Context, path: &str, tile_size: usize, rows: u8, cols: u8) -> GameResult<Self> {
        let image = Image::from_path(ctx, PathBuf::from(path).as_path())?;
        Ok(Atlas { image, aspect: Vec2::ONE, tile_size, rows, cols, rects: Vec::new() })
    }

    fn get(&self, id: u32) -> Rect {
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

    pub fn calc_aspect(&mut self, world: &World) -> () {
        self.aspect = Vec2::splat(world.tile_size as f32 / self.tile_size as f32);
    }
}
