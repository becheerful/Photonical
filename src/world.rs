use ggez::glam::UVec2;
use hecs::Entity;

use crate::energy::EnergyMaster;

pub struct BlockType(pub u32);
pub struct Position(pub UVec2);
pub struct Table(pub Option<mlua::RegistryKey>);

pub struct PowerProducer(pub u32);
pub struct PowerConsumer(pub u32);
pub struct NetworkId(pub usize);


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

    pub fn em_remove(&mut self, entity: Entity) {
        if let Ok(net_id) = self.ecs.get::<&NetworkId>(entity) {
            if let Some(network) = self.energy_master.networks.get_mut(net_id.0) {
                // damn borrow checker won't let me use `query_one_mut`
                let mut query = self.ecs.query_one::<(Option<&PowerProducer>, Option<&PowerConsumer>)>(entity).unwrap();
                let (power, demand) = query.get().unwrap();

                if power.is_some() {
                    network.imbalance -= power.unwrap().0 as i64;
                } else if demand.is_some() {
                    network.imbalance += demand.unwrap().0 as i64;
                } else {
                    network.storages -= 1;
                }
            }
        }
    }
}
