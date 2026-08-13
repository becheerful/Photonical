use crate::energy::EnergyMaster;

#[derive(Debug, Clone)]
pub struct UV(pub ggez::graphics::Rect);

#[derive(Debug, Clone, Copy)]
pub struct BlockType(pub u32);

#[derive(Debug, Clone)]
pub struct Position(pub u16, pub u16);

impl Position {
    pub fn to_vec2(&self) -> ggez::glam::Vec2 {
        ggez::glam::Vec2::new(self.0 as f32, self.1 as f32)
    }
}

pub struct Table(pub Option<mlua::RegistryKey>);

pub trait EnergyComponent {
    fn add_to_energy_master(&self, net_id: u32, energy_master: &mut EnergyMaster);

    fn get_energy_param_name() -> &'static str;

    fn get_network_mask() -> u8;
}

pub struct PowerProducer(pub f32);

impl EnergyComponent for PowerProducer {
    fn add_to_energy_master(&self, net_id: u32, energy_master: &mut EnergyMaster) {
        energy_master.add_energy_interactor(net_id, self.0);
    }

    fn get_energy_param_name() -> &'static str {
        crate::PARAM_ENERGY_POWER
    }

    fn get_network_mask() -> u8 {
        crate::NETWORK_MASK_PRODUCER
    }
}

pub struct PowerConsumer(pub f32);

impl EnergyComponent for PowerConsumer {
    fn add_to_energy_master(&self, net_id: u32, energy_master: &mut EnergyMaster) {
        energy_master.add_energy_interactor(net_id, -self.0);
    }

    fn get_energy_param_name() -> &'static str {
        crate::PARAM_ENERGY_DEMAND
    }

    fn get_network_mask() -> u8 {
        crate::NETWORK_MASK_CONSUMER
    }
}

// no component for power storage because it will be useless

pub struct NetworkId(pub u32);

pub type ECS = hecs::World;
