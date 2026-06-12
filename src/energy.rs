use hecs::Entity;

pub struct Network {
    pub producers: Vec<Entity>,
    pub consumers: Vec<Entity>,
    pub imbalance: i64,
}

impl Network {
    pub fn new() -> Self {
        Self {
            producers: Vec::new(),
            consumers: Vec::new(),
            imbalance: 0,
        }
    }

    pub fn calc_balance(&mut self, world: &hecs::World) {
        let mut total_production: u32 = 0;
        let mut total_consumption: u32 = 0;

        for producer in self.producers.iter() {
            total_production += world.get::<&crate::world::PowerProducer>(*producer).unwrap().0;
        }

        for consumer in self.consumers.iter() {
            total_consumption += world.get::<&crate::world::PowerConsumer>(*consumer).unwrap().0;
        }

        self.imbalance = total_production as i64 - total_consumption as i64;
    }
}

pub struct EnergyMaster {
    pub networks: Vec<Network>,
}

impl EnergyMaster {
    pub fn new() -> Self {
        Self { networks: Vec::new() }
    }

    pub fn update_networks(&mut self, world: &hecs::World) {
        for network in self.networks.iter_mut() {
            network.calc_balance(world);
        }
    }

    pub fn add_producer(&mut self, entity: Entity) {
        if let Some(n) = self.networks.get_mut(0) {
            n.producers.push(entity);
        } else {
            self.networks.push(Network::new());
            self.networks.get_mut(0).unwrap().producers.push(entity);
        }
    }

    pub fn add_consumer(&mut self, entity: Entity) {
        if let Some(n) = self.networks.get_mut(0) {
            n.consumers.push(entity);
        } else {
            self.networks.push(Network::new());
            self.networks.get_mut(0).unwrap().consumers.push(entity);
        }
    }
}
