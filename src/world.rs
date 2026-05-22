use ggez::glam::{UVec2, Vec2};

use crate::defs::registry;

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u32,
    pub pos: UVec2,
}

impl Block {
    pub fn new(id: u32, pos: UVec2) -> Self {
        Block { id, pos }
    }
}

pub struct World {
    pub width: u16,
    pub height: u16,
    pub bounds: Vec2,
    pub tile_size: f32,
    pub map: Vec<Block>,
    pub mechanisms: Vec<u32>,
}

impl World {
    pub fn new(width: u16, height: u16, tile_size: f32) -> Self {
        let size = width as u32 * height as u32;
        let map = (0..size).map(|i| Block::new(
            /*
                Nobody doesn't really know which block is indexed by 0.
                It can be just a stone or a nuclear bomb.
                I hope I spell the identifiers correctly...
                                            ...or something bad will happen. 💀

                TODO: rewrite for the safety of the world
            */
            registry().get_block_index("photonical:stone").unwrap_or(0),
            UVec2::new(i % (width as u32) , i / (width as u32)),
        )).collect();
        let width_pixels = (width as f32) * tile_size * 0.75;
        let height_pixels = (height as f32) * tile_size * 0.75;
        World {
            width,
            height,
            bounds: Vec2::new(width_pixels, height_pixels),
            tile_size,
            map,
            mechanisms: Vec::new(),
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&Block> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.map.get(idx as usize)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Block> {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.map.get_mut(idx as usize)
        } else {
            None
        }
    }
}
