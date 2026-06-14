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
        self.imbalance / (self.storages as i64)
    }
}

pub struct EnergyMaster {
    pub networks: Vec<Network>,
}

impl EnergyMaster {
    pub fn new() -> Self {
        Self { networks: Vec::new() }
    }

    pub fn add_producer(&mut self, net_id: usize, power: i64) {
        if let Some(n) = self.networks.get_mut(net_id) {
            n.imbalance += power;
        } else {
            self.networks.push(Network::new());
            let len = self.networks.len() - 1;
            self.networks.get_mut(len).unwrap().imbalance += power;
        }
    }

    pub fn add_consumer(&mut self, net_id: usize, demand: i64) {
        if let Some(n) = self.networks.get_mut(net_id) {
            n.imbalance -= demand;
        } else {
            self.networks.push(Network::new());
            let len = self.networks.len() - 1;
            self.networks.get_mut(len).unwrap().imbalance -= demand;
        }
    }

    pub fn add_storage(&mut self, net_id: usize) {
        if let Some(n) = self.networks.get_mut(net_id) {
            n.storages += 1;
        } else {
            self.networks.push(Network::new());
            let len = self.networks.len() - 1;
            self.networks.get_mut(len).unwrap().storages += 1;
        }
    }
}
