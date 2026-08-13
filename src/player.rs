use ggez::{glam::Vec2, input::keyboard::KeyCode};

use crate::{game::SharedData, settings::Settings, ui::PlayerUI, world::GridMap};

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec2,
    pub movement_speed: f32,
    pub screen_bounds: Vec2,
    pub directions: [bool; 4],
}

impl Camera {
    const UP: usize = 0;
    const LEFT: usize = 1;
    const DOWN: usize = 2;
    const RIGHT: usize = 3;

    pub fn new(map: &GridMap, settings: &Settings) -> Self {
        Camera {
            pos: Vec2::ZERO,
            movement_speed: 5.0,
            screen_bounds: Vec2::new(
                map.absolute_width - settings.screen_width,
                map.absolute_height - settings.screen_height,
            ),
            directions: [false; 4],
        }
    }

    pub fn update(&mut self) {
        let mut dir = Vec2::ZERO;

        if self.directions[Self::UP] {
            dir.y -= 1.0;
        }

        if self.directions[Self::LEFT] {
            dir.x -= 1.0;
        }

        if self.directions[Self::DOWN] {
            dir.y += 1.0;
        }

        if self.directions[Self::RIGHT] {
            dir.x += 1.0;
        }

        self.pos = (self.pos + dir.normalize_or_zero() * self.movement_speed)
            .clamp(Vec2::ZERO, self.screen_bounds);
    }

    pub fn resize_event(
        &mut self,
        map: &GridMap,
        _data: &SharedData,
        new_width: f32,
        new_height: f32,
    ) {
        self.screen_bounds.x = map.absolute_width - new_width;
        self.screen_bounds.y = map.absolute_height - new_height;
    }

    pub fn key_down_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::W => self.directions[Self::UP] = true,
            KeyCode::A => self.directions[Self::LEFT] = true,
            KeyCode::S => self.directions[Self::DOWN] = true,
            KeyCode::D => self.directions[Self::RIGHT] = true,
            _ => {}
        }
    }

    pub fn key_up_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::W => self.directions[Self::UP] = false,
            KeyCode::A => self.directions[Self::LEFT] = false,
            KeyCode::S => self.directions[Self::DOWN] = false,
            KeyCode::D => self.directions[Self::RIGHT] = false,
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
        atlas: &crate::res::Atlas,
        settings: &Settings,
    ) -> Self {
        Self {
            camera: Camera::new(&world.map, settings),
            ui: PlayerUI::new(registry, atlas, world.aspect, settings),
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
