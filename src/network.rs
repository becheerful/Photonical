use std::collections::HashMap;

use crate::{
    ecs::{
        Ecs, EnergyComponent, LightProperties, NetNode, PowerConsumer, PowerProducer, PowerStorage,
    },
    world::World,
};

pub const PLUG: u32 = 0;

pub struct Network {
    pub storages: u32,
    pub imbalance: f32,
}

impl Network {
    pub fn new() -> Self {
        Self {
            storages: 0,
            imbalance: 0.0,
        }
    }

    /// Returns the amount of energy that *one* storage should give up or consume.
    pub fn get_storage_imbalance(&self) -> f32 {
        if self.storages > 0 {
            self.imbalance / (self.storages as f32)
        } else {
            self.imbalance
        }
    }
}

struct NetworkDsu {
    parent: Vec<u32>,
    rank: Vec<u8>,
}

impl NetworkDsu {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size as u32).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, x: u32) -> u32 {
        if self.parent[x as usize] != x {
            self.parent[x as usize] = self.find(self.parent[x as usize]);
        }

        self.parent[x as usize]
    }

    fn union(&mut self, x: u32, y: u32) {
        let rx = self.find(x);
        let ry = self.find(y);

        if rx == ry {
            return;
        }

        match self.rank[rx as usize].cmp(&self.rank[ry as usize]) {
            std::cmp::Ordering::Less => self.parent[rx as usize] = ry,
            std::cmp::Ordering::Greater => self.parent[ry as usize] = rx,
            std::cmp::Ordering::Equal => {
                self.parent[ry as usize] = rx;
                self.rank[rx as usize] += 1;
            }
        }
    }
}

pub fn rebuild_networks(world: &mut World, ecs: &mut Ecs) {
    let mut entity_to_coord: HashMap<hecs::Entity, Vec<(u16, u16)>> = HashMap::new();
    let mut coord_to_entity = HashMap::new();
    let mut light_props = HashMap::new();

    for (idx, entity_opt) in world.map.block_entities.iter().enumerate() {
        let Some(entity) = *entity_opt else {
            continue;
        };

        let has_energy = ecs.get::<&PowerProducer>(entity).is_ok()
            || ecs.get::<&PowerConsumer>(entity).is_ok()
            || ecs.get::<&PowerStorage>(entity).is_ok();

        if has_energy && let Ok(props) = ecs.get::<&LightProperties>(entity) {
            let x = idx as u16 % world.map.width;
            let y = idx as u16 / world.map.width;

            entity_to_coord.entry(entity).or_default().push((x, y));
            coord_to_entity.insert((x, y), entity);
            light_props.insert(entity, props.wavelength);
        }
    }

    let mut energy_entities = Vec::new();

    for (&entity, _) in &entity_to_coord {
        energy_entities.push(entity);
    }

    if energy_entities.is_empty() {
        return;
    }

    let mut dsu = NetworkDsu::new(energy_entities.len());
    let mut entity_to_idx = HashMap::new();
    for (i, &entity) in energy_entities.iter().enumerate() {
        entity_to_idx.insert(entity, i as u32);
    }

    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    for entity in &energy_entities {
        let cur_idx = entity_to_idx[entity];
        let coord = entity_to_coord.get(entity).unwrap();
        let wavelength = light_props[entity];

        for &(x, y) in coord {
            for &(dx, dy) in &directions {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= world.map.width as i32 || ny >= world.map.height as i32
                {
                    continue;
                }

                if coord_to_entity.get(&(nx as u16, ny as u16)) == Some(entity) {
                    continue;
                }

                let mut cx = nx;
                let mut cy = ny;
                let mut steps = 0;

                while steps < crate::settings::CONNECTION_RADIUS
                    && cx >= 0
                    && cy >= 0
                    && cx < world.map.width as i32
                    && cy < world.map.height as i32
                {
                    let cur_x = cx as u16;
                    let cur_y = cy as u16;

                    if let Some(&other_entity) = coord_to_entity.get(&(cur_x, cur_y)) {
                        if other_entity == *entity {
                            cx += dx;
                            cy += dy;
                            continue;
                        }

                        let other_wavelength = ecs
                            .get::<&LightProperties>(other_entity)
                            .unwrap()
                            .wavelength;
                        if other_wavelength == wavelength {
                            let other_idx = entity_to_idx[&other_entity];
                            dsu.union(cur_idx, other_idx);
                        }

                        break;
                    }

                    cx += dx;
                    cy += dy;
                    steps += 1;
                }
            }
        }
    }

    let mut network_counter: u32 = 0;
    let mut root_to_network = HashMap::new();

    for &entity in &energy_entities {
        let root = dsu.find(entity_to_idx[&entity]);
        ecs.get::<&mut NetNode>(entity).unwrap().0 =
            *root_to_network.entry(root).or_insert_with(|| {
                let id = network_counter;
                network_counter += 1;
                id
            });
    }

    collect_network_stats(world, ecs);
}

pub fn collect_network_stats(world: &mut World, ecs: &mut Ecs) {
    let mut networks = HashMap::new();

    for (_, (nid, prod)) in ecs.query_mut::<(&NetNode, &PowerProducer)>() {
        prod.apply_to_network(nid.0, &mut networks);
    }

    for (_, (nid, cons)) in ecs.query_mut::<(&NetNode, &PowerConsumer)>() {
        cons.apply_to_network(nid.0, &mut networks);
    }

    for (_, (nid, storage)) in ecs.query_mut::<(&NetNode, &PowerStorage)>() {
        storage.apply_to_network(nid.0, &mut networks);
    }

    world.networks = networks;
}
