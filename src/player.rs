use ggez::{glam::Vec2, input::keyboard::KeyCode};

use crate::{entity::Entity, world::BlockType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Test = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Item(ItemType),
    Block(BlockType),
}

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub stack_max: u8,
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

    pub fn move_towards(&mut self, dir: Vec2, bounds: Vec2) {
        self.pos = (self.pos + dir * self.movement_speed)
            .max(Vec2::ZERO)
            .min(bounds);
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub camera: Camera,
    pub entity: Entity,
    pub inventory: Vec<Option<Item>>,
}

impl Player {
    pub fn new(max_health: u16, slot_count: usize) -> Self {
        Player {
            camera: Camera::new(),
            entity: Entity::new(max_health),
            inventory: vec![None; slot_count]
        }
    }

    pub fn add_item(&mut self, new_item: Item) -> bool {
        for slot in self.inventory.iter_mut() {
            if let Some(item) = slot {
                if item.kind == new_item.kind {
                    if item.count + new_item.count > item.stack_max {
                        continue;
                    }

                    item.count += new_item.count;
                    return true;
                }
            }
        }

        for slot in self.inventory.iter_mut() {
            if slot.is_none() {
                *slot = Some(new_item);
                return true;
            }
        }

        false
    }
}
