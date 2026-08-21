use ggez::{GameError, GameResult};

use crate::defs::BlockDef;

pub mod fields {
    // parameter names for .json block definitions
    pub const ENERGY_POWER: &str = "power";
    pub const ENERGY_DEMAND: &str = "demand";
    pub const WAVELENGTH: &str = "wavelength";
    pub const ENERGY_MASK: &str = "mask";
}

pub fn get_energy_interaction_value<T: crate::ecs::EnergyComponent>(
    bd: &BlockDef,
) -> GameResult<f32> {
    let parameter_name = T::get_energy_param_name().ok_or(GameError::CustomError(format!(
        "No associated parameter for network mask {}",
        T::get_network_mask()
    )))?;

    Ok(bd
        .net
        .get(&parameter_name)
        .ok_or(GameError::ConfigError(format!(
            "Missing parameter `{parameter_name}` for network mask {}",
            T::get_network_mask()
        )))?
        .as_f64()
        .ok_or(GameError::CustomError(format!(
            "Invalid value for `{parameter_name}`"
        )))? as f32)
}

pub fn get_wavelength(bd: &BlockDef) -> GameResult<u16> {
    Ok(bd
        .net
        .get(fields::WAVELENGTH)
        .ok_or(GameError::ConfigError(
            "Missing parameter `wavelength`".to_owned(),
        ))?
        .as_u64()
        .ok_or(GameError::ConfigError(
            "Invalid value for `wavelength`".to_owned(),
        ))? as u16)
}
