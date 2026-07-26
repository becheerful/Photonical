use ggez::glam::UVec2;
use hecs::Entity;

use crate::{
    ecs::{BlockType, ECS, NetworkId, Position, PowerConsumer, PowerProducer},
    energy::EnergyMaster,
};

pub struct World {
    pub map: GridMap,
    pub energy_master: EnergyMaster,
    pub aspect: ggez::glam::Vec2,
}

impl World {
    pub fn new(registry: &crate::defs::Registry, width: u16, height: u16, tile_size: f32) -> Self {
        Self {
            map: GridMap::new(registry, width, height, tile_size),
            energy_master: EnergyMaster::new(),
            aspect: ggez::glam::Vec2::splat(tile_size / crate::TEXTURE_SIZE),
        }
    }

    pub fn remove_entity(&mut self, ecs: &mut ECS, mut x: u16, mut y: u16) {
        if let Some(entity) = self.map.get(x, y) {
            let mut block_type = 0;

            if let Ok((id, pos, net, producer, consumer)) = ecs.query_one_mut::<(
                &BlockType,
                &Position,
                Option<&NetworkId>,
                Option<&PowerProducer>,
                Option<&PowerConsumer>,
            )>(entity)
            {
                block_type = id.0;
                x = pos.0;
                y = pos.1;

                if let Some(n) = net {
                    let network = self.energy_master.networks.get_mut(&n.0).unwrap();

                    if let Some(power) = producer {
                        network.imbalance -= power.0;
                    } else if let Some(demand) = consumer {
                        network.imbalance += demand.0;
                    } else {
                        network.storages -= 1;
                    }

                    if network.is_empty() {
                        self.energy_master.networks.remove(&n.0);
                    }
                }
            }

            if let Err(e) = ecs.despawn(entity) {
                eprintln!("{e}");
            }

            let size = crate::defs::registry()
                .get_block_directly(block_type)
                .unwrap()
                .size;
            for col in x..(x + size) {
                for row in y..(y + size) {
                    let index = self.map.index(col, row);
                    self.map.block_entities[index] = None;
                }
            }
        }
    }

    /// Check if there is a free space for a block. If there is, it returns `true`; otherwise, it returns `false`.
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

    pub fn place_block(&mut self, x: u16, y: u16, size: u16, e: Option<Entity>) {
        for col in x..(x + size) {
            for row in y..(y + size) {
                let index = self.map.index(col, row);
                self.map.block_entities[index] = e.clone();
            }
        }
    }
}

pub struct GridMap {
    pub width: u16,
    pub height: u16,
    pub tile_size: f32,
    pub static_tiles: Vec<(u32, UVec2)>,
    pub block_entities: Vec<Option<Entity>>,
}

impl GridMap {
    pub fn new(registry: &crate::defs::Registry, width: u16, height: u16, tile_size: f32) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            tile_size,
            static_tiles: (0..size)
                .map(|i| {
                    (
                        registry.get_block_index("photonical:sand").unwrap(),
                        UVec2::new(i as u32 % width as u32, i as u32 / width as u32),
                    )
                })
                .collect(),
            block_entities: vec![None; size],
        }
    }

    pub fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Entity> {
        self.block_entities[self.index(x, y)]
    }
}
