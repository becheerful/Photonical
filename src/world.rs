use ggez::{GameResult, glam::Vec2};

use crate::defs;

#[derive(Debug, Clone)]
pub struct Block {
    pub def: defs::BlockDef,
    pub pos: Vec2,
}

impl Block {
    pub fn new(def: defs::BlockDef, pos: Vec2) -> Self {
        Block { def, pos }
    }
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub bounds: Vec2,
    pub tile_size: usize,
    pub map: Vec<Block>,
}

impl World {
    pub fn new(width: usize, height: usize, tile_size: usize) -> Self {
        let size = (width * height) as usize;
        let map = (0..size).map(|i| Block::new(
            defs::get_block("photonical:stone").unwrap(),
            Vec2::new(((i % width) * tile_size) as f32, ((i / width) * tile_size) as f32),
        )).collect();
        let width_pixels = (width * tile_size) as f32 * 0.75;
        let height_pixels = (height * tile_size) as f32 * 0.75;
        World {
            width,
            height,
            bounds: Vec2::new(width_pixels, height_pixels),
            tile_size,
            map,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Block> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.map.get(idx)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Block> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.map.get_mut(idx)
        } else {
            None
        }
    }

    pub fn update(&mut self) -> GameResult {
        Ok(())
    }
}
