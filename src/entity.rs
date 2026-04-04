#[derive(Debug, Clone)]
pub struct Entity {
    pub health: u16,
    pub max_health: u16,
}

impl Entity {
    pub fn new(max_health: u16) -> Self {
        Entity { health: max_health, max_health }
    }
}
