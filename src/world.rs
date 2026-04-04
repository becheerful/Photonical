use ggez::{GameResult, glam::Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air = 0,
    Stone,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub id: BlockType,
    pub pos: Vec2,
}

impl Block {
    pub fn new(id: BlockType, pos: Vec2) -> Self {
        Block { id, pos }
    }
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub tile_size: usize,
    pub map: Vec<Block>,
}

impl World {
    pub fn new(width: usize, height: usize, tile_size: usize) -> Self {
        let size = (width * height) as usize;
        let map = (0..size).map(|i| Block::new(
            BlockType::Stone, Vec2::new(((i % width) * tile_size) as f32, ((i / width) * tile_size) as f32),
        )).collect();
        World { width, height, tile_size, map }
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
