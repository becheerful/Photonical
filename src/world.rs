use std::collections::HashMap;

use ggez::{GameResult, glam::UVec2};
use hecs::Entity;

use crate::{
    defs::registry,
    ecs::{BlockType, Ecs, Position},
    network::Network,
    res::TEXTURE_SIZE,
};

pub struct World {
    pub map: GridMap,
    pub networks: HashMap<u32, Network>,
    pub aspect: f32,
    pub connections: Vec<crate::ecs::LightBeam>,
}

impl World {
    pub fn new(width: u16, height: u16) -> GameResult<Self> {
        Ok(Self {
            map: GridMap::new(width, height)?,
            networks: HashMap::new(),
            aspect: 1.0,
            connections: Vec::new(),
        })
    }

    pub fn remove_entity(&mut self, ecs: &mut Ecs, mut x: u16, mut y: u16) -> GameResult {
        if let Some(entity) = self.map.get(x, y) {
            let mut block_type = 0;

            if let Ok((id, pos)) = ecs.query_one_mut::<(&BlockType, &Position)>(entity) {
                block_type = id.0;
                x = pos.0; // i don't know what's going on here, so i won't touch anything
                y = pos.1;
            }

            if let Err(e) = ecs.despawn(entity) {
                eprintln!("{e}");
            }

            let size = registry().get_block_directly(block_type)?.size;
            for col in x..(x + size) {
                for row in y..(y + size) {
                    let index = self.map.index(col, row);
                    self.map.block_entities[index] = None;
                }
            }
        }

        Ok(())
    }

    pub fn check_for_space(&self, x: u16, y: u16, size: u16) -> bool {
        if x + size > self.map.width || y + size > self.map.height {
            return false;
        }

        for col in x..(x + size) {
            for row in y..(y + size) {
                if self.map.get(col, row).is_some() {
                    return false;
                }
            }
        }

        true
    }

    pub fn place_block(&mut self, x: u16, y: u16, size: u16, e: Entity) {
        for col in x..(x + size) {
            for row in y..(y + size) {
                let index = self.map.index(col, row);
                self.map.block_entities[index] = Some(e);
            }
        }
    }
}

pub struct GridMap {
    pub width: u16,
    pub height: u16,
    pub absolute_width: f32,
    pub absolute_height: f32,
    pub static_tiles: Vec<(u32, UVec2)>,
    pub block_entities: Vec<Option<Entity>>,
}

impl GridMap {
    pub fn new(width: u16, height: u16) -> GameResult<Self> {
        Ok(Self {
            width,
            height,
            absolute_width: width as f32 * TEXTURE_SIZE,
            absolute_height: height as f32 * TEXTURE_SIZE,
            static_tiles: GridMap::generate_world(width, height)?,
            block_entities: vec![None; width as usize * height as usize],
        })
    }

    fn generate_world(width: u16, height: u16) -> GameResult<Vec<(u32, UVec2)>> {
        let mut map = Vec::with_capacity(width as usize * height as usize);
        for i in 0..map.capacity() {
            map.push((
                registry().get_block_index("photonical:sand")?,
                UVec2::new(i as u32 % width as u32, i as u32 / width as u32),
            ));
        }

        map[0] = (
            registry().get_block_index("photonical:diamond_placer")?,
            UVec2::splat(0),
        );

        Ok(map)
    }

    pub fn index(&self, x: u16, y: u16) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Entity> {
        self.block_entities[self.index(x, y)]
    }
}
