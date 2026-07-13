use ggez::glam::UVec2;
use hecs::Entity;

use crate::energy::EnergyMaster;

pub struct BlockType(pub u32);
pub struct Position(pub UVec2);
pub struct Table(pub Option<mlua::RegistryKey>);

pub struct PowerProducer(pub u32);
pub struct PowerConsumer(pub u32);
// first `u32` is for stored energy, second `u32` is for capacity
pub struct PowerStorage(pub u32, pub u32);
pub struct NetworkId(pub u32);


pub struct World {
    pub map: GridMap,
    pub ecs: hecs::World,
    pub energy_master: EnergyMaster,
    pub aspect: ggez::glam::Vec2,
}

impl World {
    pub fn new(registry: &crate::defs::Registry, width: u16, height: u16, tile_size: f32) -> Self {
        Self {
            map: GridMap::new(registry, width, height, tile_size),
            ecs: hecs::World::new(),
            energy_master: EnergyMaster::new(),
            aspect: ggez::glam::Vec2::splat(tile_size / crate::TEXTURE_SIZE),
        }
    }

    pub fn update(&mut self) {
        self.energy_master.update(&mut self.ecs);
    }

    pub fn remove_entity(&mut self, x: u16, y: u16) {
        if let Some(entity) = self.map.block_entities[self.map.index(x, y)] {
            let mut block_type = 0;

            if let Ok((id, net, producer, consumer)) = self.ecs.query_one_mut::<(
                &BlockType, Option<&NetworkId>, Option<&PowerProducer>, Option<&PowerConsumer>
            )>(entity) {
                block_type = id.0;

                if let Some(n) = net {
                    let network = self.energy_master.networks.get_mut(&n.0).unwrap();

                    if let Some(power) = producer {
                        network.imbalance -= power.0 as i64;
                    } else if let Some(demand) = consumer {
                        network.imbalance += demand.0 as i64;
                    } else {
                        network.storages -= 1;
                    }

                    if network.is_empty() {
                        self.energy_master.networks.remove(&n.0);
                    }
                }
            }

            if let Err(e) = self.ecs.despawn(entity) {
                eprintln!("{e}");
            }

            let size = crate::defs::registry().get_block_directly(block_type).unwrap().size;
            for col in x..(x + size) {
                for row in y..(y + size) {
                    let index = self.map.index(col, row);
                    self.map.block_entities[index] = None;
                }
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
            static_tiles: (0..size).map(|i| (
                registry.get_block_index("photonical:sand").unwrap(),
                UVec2::new(i as u32 % width as u32, i as u32 / width as u32),
            )).collect(),
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
