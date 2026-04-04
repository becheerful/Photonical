use crate::{entity::Entity, world::BlockType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Test = 0,
}

#[derive(Debug, Clone, Copy)]
pub enum ItemKind {
    Item(ItemType),
    Block(BlockType),
}

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub count: u8,
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub entity: Entity,
    pub inventory: Vec<Item>,
}

impl Player {
    pub fn new(max_health: u16) -> Self {
        Player { entity: Entity::new(max_health), inventory: Vec::new() }
    }
}
