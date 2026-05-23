use ggez::{glam::Vec2, input::keyboard::KeyCode};

use crate::{Settings, world::World};

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec2,
    pub movement_speed: f32,
    pub screen_bounds: Vec2,
}

impl Camera {
    pub fn new(world: &World, settings: &Settings) -> Self {
        Camera {
            pos: Vec2::ZERO,
            movement_speed: 10.0,
            screen_bounds: Vec2::new(
                world.width as f32 * world.tile_size - settings.sc_width,
                world.height as f32 * world.tile_size - settings.sc_height,
            ),
        }
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

    pub fn move_towards(&mut self, dir: Vec2) {
        self.pos = (self.pos + dir * self.movement_speed)
            .max(Vec2::ZERO)
            .min(self.screen_bounds);
    }
}
