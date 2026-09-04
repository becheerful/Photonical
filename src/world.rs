use std::collections::HashMap;

use ggez::GameResult;
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
    pub zoom: f32,
    pub block_entities: Vec<Option<Entity>>,
    pub connections: Vec<crate::ecs::LightBeam>,
}

impl World {
    pub fn new(ecs: &mut Ecs, width: u16, height: u16) -> GameResult<Self> {
        Ok(Self {
            map: GridMap::new(ecs, width, height)?,
            networks: HashMap::new(),
            zoom: 1.0,
            block_entities: vec![None; width as usize * height as usize],
            connections: Vec::new(),
        })
    }

    pub fn remove_entity(&mut self, ecs: &mut Ecs, mut x: u16, mut y: u16) -> GameResult {
        if let Some(entity) = self.get(x, y) {
            let mut block_type = 0;

            if let Ok((id, pos)) = ecs.query_one_mut::<(&BlockType, &Position)>(entity) {
                block_type = id.0;
                /*
                 * Finds the top-left corner of the block in order
                 * to correctly delete it from all the tiles it occupies.
                 */
                x = pos.0;
                y = pos.1;
            }

            if let Err(e) = ecs.despawn(entity) {
                eprintln!("{e}");
            }

            let size = registry().get_block_directly(block_type)?.size;
            for col in x..(x + size) {
                for row in y..(y + size) {
                    *self.get_mut(col, row) = None;
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
                if self.get(col, row).is_some() {
                    return false;
                }
            }
        }

        true
    }

    pub fn place_block(&mut self, x: u16, y: u16, size: u16, e: Entity) {
        for col in x..(x + size) {
            for row in y..(y + size) {
                *self.get_mut(col, row) = Some(e);
            }
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Entity> {
        self.block_entities[self.map.index(x, y)]
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Option<Entity> {
        &mut self.block_entities[self.map.index(x, y)]
    }

    /// Searches for an entity in `block_entities`;
    /// if the entity is not found, it returns an entity from `tiles`.
    pub fn get_any(&self, x: u16, y: u16) -> Entity {
        let index = self.map.index(x, y);
        self.block_entities[index].unwrap_or(self.map.tiles[index])
    }
}

pub struct GridMap {
    pub width: u16,
    pub height: u16,
    pub absolute_width: f32,
    pub absolute_height: f32,
    pub tiles: Vec<Entity>,
}

impl GridMap {
    pub fn new(ecs: &mut Ecs, width: u16, height: u16) -> GameResult<Self> {
        Ok(Self {
            width,
            height,
            absolute_width: width as f32 * TEXTURE_SIZE,
            absolute_height: height as f32 * TEXTURE_SIZE,
            tiles: GridMap::generate_world(ecs, width, height)?,
        })
    }

    pub fn replace_tile(&mut self, index: usize, ecs: &mut Ecs, new_id: u32) -> Option<Entity> {
        let e = self.tiles[index];

        match ecs.get::<&mut BlockType>(e) {
            Ok(mut id) => {
                if id.0 == new_id {
                    return None;
                }

                id.0 = new_id;
            }

            Err(e) => {
                eprintln!("{e}");
            }
        }

        Some(e)
    }

    fn generate_world(ecs: &mut Ecs, width: u16, height: u16) -> GameResult<Vec<Entity>> {
        let sand_index = registry().get_block_index("photonical:sand")?;
        Ok(ecs
            .spawn_batch((0..(width as usize * height as usize)).map(|i| {
                (
                    BlockType(sand_index),
                    Position(i as u16 % width, i as u16 / width),
                )
            }))
            .collect::<Vec<Entity>>())
    }

    pub fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: u16, y: u16) -> Entity {
        self.tiles[self.index(x, y)]
    }
}
