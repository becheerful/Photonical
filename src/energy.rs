use std::collections::HashMap;

pub struct Network {
    pub storages: u32,
    pub imbalance: i64,
}

impl Network {
    pub fn new() -> Self {
        Self {
            storages: 0,
            imbalance: 0,
        }
    }

    /// Returns the amount of energy that one storage should give up or consume
    pub fn get_storage_imbalance(&self) -> i64 {
        if self.storages > 0 { self.imbalance / (self.storages as i64) } else { self.imbalance }
    }

    pub fn is_empty(&self) -> bool {
        self.imbalance == 0 && self.storages == 0
    }
}

pub struct EnergyMaster {
    pub networks: HashMap<u32, Network>,
}

impl EnergyMaster {
    pub fn new() -> Self {
        Self { networks: HashMap::new() }
    }

    pub fn update(&mut self, ecs: &mut hecs::World) {
        for (_, (net_id, storage)) in ecs.query_mut::<(
            &crate::world::NetworkId, &mut crate::world::PowerStorage,
        )>() {
            let imbalance = self.networks.get(&net_id.0).unwrap().get_storage_imbalance();
            let stored = &mut storage.0;
            *stored = storage.1.min(if imbalance >= 0 {
                *stored + (imbalance.abs() as u32)
            } else {
                *stored - (imbalance.abs() as u32)
            }).max(0);
            // println!("{}", *stored);
        }
    }

    pub fn add_producer(&mut self, net_id: u32, power: i64) {
        self.networks.entry(net_id).or_insert(Network::new()).imbalance += power;
    }

    pub fn add_consumer(&mut self, net_id: u32, demand: i64) {
        self.networks.entry(net_id).or_insert(Network::new()).imbalance -= demand;
    }

    pub fn add_storage(&mut self, net_id: u32) {
        self.networks.entry(net_id).or_insert(Network::new()).storages += 1;
    }
}
