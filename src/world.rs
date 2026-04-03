use ggez::{GameResult, glam::Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Air = 0,
    Stone,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub id: TileType,
    pub pos: Vec2,
}

impl Tile {
    pub fn new(id: TileType, pos: Vec2) -> Self {
        Tile { id, pos }
    }
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub tile_size: usize,
    pub map: Vec<Tile>,
}

impl World {
    pub fn new(width: usize, height: usize, tile_size: usize) -> Self {
        let size = (width * height) as usize;
        let map = (0..size).map(|i| Tile::new(
            TileType::Air, Vec2::new(((i % width) * tile_size) as f32, ((i / width) * tile_size) as f32),
        )).collect();
        World { width, height, tile_size, map }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.map.get(idx)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
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
