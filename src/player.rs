use ggez::{glam::Vec2, input::keyboard::KeyCode};

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
pub struct Camera {
    pub pos: Vec2,
    pub movement_speed: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera { pos: Vec2::ZERO, movement_speed: 8.0 }
    }

    pub fn handle_movement(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::W => self.pos.y -= self.movement_speed,
            KeyCode::A => self.pos.x -= self.movement_speed,
            KeyCode::S => self.pos.y += self.movement_speed,
            KeyCode::D => self.pos.x += self.movement_speed,
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub camera: Camera,
    pub entity: Entity,
    pub inventory: Vec<Item>,
}

impl Player {
    pub fn new(max_health: u16) -> Self {
        Player { camera: Camera::new(), entity: Entity::new(max_health), inventory: Vec::new() }
    }
}
