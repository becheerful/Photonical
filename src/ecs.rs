use std::collections::HashMap;

use crate::network::Network;

#[derive(Debug, Clone, Copy)]
pub struct UV(pub ggez::graphics::Rect);

#[derive(Debug, Clone, Copy)]
pub struct BlockType(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Position(pub u16, pub u16);

impl Position {
    pub fn to_vec2(&self) -> ggez::glam::Vec2 {
        ggez::glam::Vec2::new(self.0 as f32, self.1 as f32)
    }

    pub fn to_uvec2(&self) -> ggez::glam::UVec2 {
        ggez::glam::UVec2::new(self.0 as u32, self.1 as u32)
    }
}

pub struct Table(pub Option<mlua::RegistryKey>);

pub trait EnergyComponent {
    fn apply_to_network(&self, net_id: u32, networks: &mut HashMap<u32, Network>);

    fn get_energy_param_name() -> Option<String>;

    fn get_network_mask() -> u8;
}

#[derive(Debug, Clone, Copy)]
pub struct PowerProducer(pub f32);

impl EnergyComponent for PowerProducer {
    fn apply_to_network(&self, net_id: u32, networks: &mut HashMap<u32, Network>) {
        networks.entry(net_id).or_insert(Network::new()).imbalance += self.0;
    }

    fn get_energy_param_name() -> Option<String> {
        Some(crate::json::fields::ENERGY_POWER.to_owned())
    }

    fn get_network_mask() -> u8 {
        crate::network::mask::PRODUCER
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowerConsumer(pub f32);

impl EnergyComponent for PowerConsumer {
    fn apply_to_network(&self, net_id: u32, networks: &mut HashMap<u32, Network>) {
        networks.entry(net_id).or_insert(Network::new()).imbalance -= self.0;
    }

    fn get_energy_param_name() -> Option<String> {
        Some(crate::json::fields::ENERGY_DEMAND.to_owned())
    }

    fn get_network_mask() -> u8 {
        crate::network::mask::CONSUMER
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowerStorage;

impl EnergyComponent for PowerStorage {
    fn apply_to_network(&self, net_id: u32, networks: &mut HashMap<u32, Network>) {
        networks.entry(net_id).or_insert(Network::new()).storages += 1;
    }

    fn get_energy_param_name() -> Option<String> {
        None
    }

    fn get_network_mask() -> u8 {
        crate::network::mask::STORAGE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetNode(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct LightProperties(pub crate::network::LightColor);

pub type Ecs = hecs::World;
