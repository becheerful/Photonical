use ggez::glam::UVec2;
use hecs::Entity;

use crate::energy::EnergyMaster;

pub struct BlockType(pub u32);
pub struct Position(pub UVec2);
pub struct Table(pub Option<mlua::RegistryKey>);

pub struct PowerProducer(pub u32);
pub struct PowerConsumer(pub u32);


pub struct World {
    pub ecs: hecs::World,
    pub width: u16,
    pub height: u16,
    pub tile_size: f32,
    pub static_tiles: Vec<(u32, UVec2)>,
    pub block_entities: Vec<Option<Entity>>,
    pub energy_master: EnergyMaster,
}

impl World {
    pub fn new(width: u16, height: u16, tile_size: f32) -> Self {
        let size = width as usize * height as usize;
        Self {
            ecs: hecs::World::new(),
            width,
            height,
            tile_size,
            static_tiles: (0..size).map(|i| (
                crate::defs::registry().get_block_index("photonical:stone").unwrap(),
                UVec2::new(i as u32 % width as u32, i as u32 / width as u32),
            )).collect(),
            block_entities: vec![None; size],
            energy_master: EnergyMaster::new(),
        }
    }

    pub fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Entity> {
        self.block_entities[self.index(x, y)]
    }

    pub fn get_directly(&self, index: usize) -> Option<Entity> {
        self.block_entities[index]
    }

    pub fn update_networks(&mut self) {
        self.energy_master.update_networks(&self.ecs);
    }
}
