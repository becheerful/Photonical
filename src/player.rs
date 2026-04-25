use ggez::{glam::Vec2, input::keyboard::KeyCode};

use crate::{defs, entity::Entity};

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
}

impl Player {
    pub fn new(max_health: u16) -> Self {
        Player {
            camera: Camera::new(),
            entity: Entity::new(max_health)
        }
    }
}
