use ggez::{glam::Vec2, input::keyboard::KeyCode};

use crate::{Settings, game::SharedData, ui::PlayerUI, world::GridMap};

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec2,
    pub movement_speed: f32,
    pub screen_bounds: Vec2,
    pub direction: Vec2,
}

impl Camera {
    pub fn new(map: &GridMap, settings: &Settings) -> Self {
        Camera {
            pos: Vec2::ZERO,
            movement_speed: 5.0,
            screen_bounds: Vec2::new(
                map.width as f32 * map.tile_size - settings.sc_width,
                map.height as f32 * map.tile_size - settings.sc_height,
            ),
            direction: Vec2::ZERO,
        }
    }

    pub fn update(&mut self) {
        self.pos = (self.pos + self.direction * self.movement_speed)
            .max(Vec2::ZERO)
            .min(self.screen_bounds);
    }

    pub fn resize_event(
        &mut self,
        map: &GridMap,
        data: &SharedData,
        new_width: f32,
        new_height: f32,
    ) {
        self.screen_bounds.x = map.width as f32 * data.settings.tile_size - new_width;
        self.screen_bounds.y = map.height as f32 * data.settings.tile_size - new_height;
    }

    pub fn key_down_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::W => self.direction.y = -1.0,
            KeyCode::A => self.direction.x = -1.0,
            KeyCode::S => self.direction.y = 1.0,
            KeyCode::D => self.direction.x = 1.0,
            _ => {}
        }
    }

    pub fn key_up_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::W => self.direction.y = 0.0,
            KeyCode::A => self.direction.x = 0.0,
            KeyCode::S => self.direction.y = 0.0,
            KeyCode::D => self.direction.x = 0.0,
            _ => {}
        }
    }
}

pub struct Player {
    pub camera: Camera,
    pub ui: PlayerUI,
}

impl Player {
    pub fn new(
        world: &crate::world::World,
        registry: &crate::defs::Registry,
        settings: &Settings,
    ) -> Self {
        Self {
            camera: Camera::new(&world.map, settings),
            ui: PlayerUI::new(registry, &world.aspect, settings),
        }
    }

    pub fn draw(
        &self,
        canvas: &mut ggez::graphics::Canvas,
        atlas: &crate::res::Atlas,
    ) -> ggez::GameResult {
        self.ui.draw(canvas, atlas)?;
        Ok(())
    }

    pub fn resize_event(
        &mut self,
        map: &GridMap,
        data: &SharedData,
        new_width: f32,
        new_height: f32,
    ) {
        self.camera.resize_event(map, data, new_width, new_height);
        self.ui.resize_event(new_width, new_height);
    }
}
