use ggez::glam::Vec2;

use crate::defs::registry;

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u32,
    pub pos: Vec2,
}

impl Block {
    pub fn new(id: u32, pos: Vec2) -> Self {
        Block { id, pos }
    }
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub bounds: Vec2,
    pub tile_size: usize,
    pub map: Vec<Block>,
    pub mechanisms: Vec<usize>,
}

impl World {
    pub fn new(width: usize, height: usize, tile_size: usize) -> Self {
        let size = (width * height) as usize;
        let map = (0..size).map(|i| Block::new(
            /*
                Nobody doesn't really know which block is indexed by 0.
                It can be just a stone or a nuclear bomb.
                I hope I spell the identifiers correctly...
                                            ...or something bad will happen. 💀

                TODO: rewrite for the safety of the world
            */
            registry().get_block_index("photonical:stone").unwrap_or(0),
            Vec2::new((i % width) as f32, (i / width) as f32),
        )).collect();
        let width_pixels = (width * tile_size) as f32 * 0.75;
        let height_pixels = (height * tile_size) as f32 * 0.75;
        World {
            width,
            height,
            bounds: Vec2::new(width_pixels, height_pixels),
            tile_size,
            map,
            mechanisms: Vec::new(),
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
}
