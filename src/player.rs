use ggez::{glam::Vec2, input::keyboard::KeyCode, mint::Point2};

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

    pub fn get_movement_vector(&mut self, keycode: KeyCode) -> Option<Vec2> {
        match keycode {
            KeyCode::W => Some(Vec2 { x:  0.0, y: -1.0 }),
            KeyCode::A => Some(Vec2 { x: -1.0, y:  0.0 }),
            KeyCode::S => Some(Vec2 { x:  0.0, y:  1.0 }),
            KeyCode::D => Some(Vec2 { x:  1.0, y:  0.0 }),
            _ => None,
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
