use std::collections::HashMap;

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

    pub fn is_empty(&self) -> bool {
        self.imbalance == 0.0 && self.storages == 0
    }
}

pub struct EnergyMaster {
    pub networks: HashMap<u32, Network>,
}

impl EnergyMaster {
    pub fn new() -> Self {
        Self {
            networks: HashMap::new(),
        }
    }

    pub fn add_energy_interactor(&mut self, net_id: u32, delta_power: f32) {
        self.networks
            .entry(net_id)
            .or_insert(Network::new())
            .imbalance += delta_power;
    }

    pub fn add_storage(&mut self, net_id: u32) {
        self.networks
            .entry(net_id)
            .or_insert(Network::new())
            .storages += 1;
    }
}
