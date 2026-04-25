use std::path::PathBuf;

use ggez::{
    Context, ContextBuilder, GameResult,
    conf::FullscreenType,
    event::{self, EventHandler, MouseButton},
    glam::Vec2,
    graphics::{Color, DrawParam, Drawable, InstanceArray, Rect, Sampler, Text},
    input::keyboard::{KeyCode, KeyInput}
};

use crate::res::Atlas;

mod world;
mod res;
mod entity;
mod player;
mod defs;

struct Settings {
    pub aspect: Vec2,
    pub sc_width: f32,
    pub sc_height: f32,
    pub fullscreen_type: FullscreenType,
}

impl Settings {
    pub fn new() -> Self {
        Settings { aspect: Vec2::ONE, sc_width: 640.0, sc_height: 480.0, fullscreen_type: FullscreenType::Windowed }
    }
}

struct Game {
    pub world: world::World,
    pub atlas: res::Atlas,
    pub settings: Settings,
}

impl Game {
    fn new(_ctx: &mut Context, settings: Settings, atlas: res::Atlas, world: world::World) -> GameResult<Game> {
        Ok(Game {
            settings,
            world,
            atlas,
        })
    }
}

impl EventHandler for Game {
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeated: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            match keycode {
                KeyCode::F11 => {     
                    self.settings.fullscreen_type = match self.settings.fullscreen_type {
                        FullscreenType::Windowed => FullscreenType::Desktop,
                        _ => FullscreenType::Windowed,
                    };

                    ctx.gfx.set_fullscreen(self.settings.fullscreen_type)?;
                }
                KeyCode::Escape => ctx.request_quit(),
                key => {
                    if let Some(direction) = self.world.player.camera.get_movement_vector(key) {
                        self.world.player.camera.move_towards(direction, self.world.bounds);
                    }
                }
            }
        }
    
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.mouse.button_pressed(MouseButton::Right) {
            let mouse_point = ctx.mouse.position();
            let rel_x = mouse_point.x + self.world.player.camera.pos.x;
            let rel_y = mouse_point.y + self.world.player.camera.pos.y;

            let tile_x = rel_x / self.world.tile_size as f32;
            let tile_y = rel_y / self.world.tile_size as f32;

            if 0.0 <= mouse_point.x && mouse_point.x < self.settings.sc_width && 0.0 <= mouse_point.y && mouse_point.y < self.settings.sc_height {
                let tile = self.world.get_mut(tile_x as usize, tile_y as usize).unwrap();
                tile.id = defs::get_block("advent:stone").unwrap();
            }
        } else if ctx.mouse.button_pressed(MouseButton::Left) {
            let mouse_point = ctx.mouse.position();
            let rel_x = mouse_point.x + self.world.player.camera.pos.x;
            let rel_y = mouse_point.y + self.world.player.camera.pos.y;

            let tile_x = rel_x / self.world.tile_size as f32;
            let tile_y = rel_y / self.world.tile_size as f32;

            if 0.0 <= mouse_point.x && mouse_point.x < self.settings.sc_width && 0.0 <= mouse_point.y && mouse_point.y < self.settings.sc_height {
                let tile = self.world.get_mut(tile_x as usize, tile_y as usize).unwrap();
                tile.id = defs::get_block("advent:air").unwrap();
            }
        }

        self.world.update()?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, Color::WHITE);
        canvas.set_sampler(Sampler::nearest_clamp());

        let mut array = InstanceArray::new(ctx, self.atlas.image.clone());

        for tile in &self.world.map {
            let rect = self.atlas.get_uv(&tile.id);
            array.push(DrawParam::default().src(*rect).dest(tile.pos - self.world.player.camera.pos).scale(self.settings.aspect));
        }

        array.draw(&mut canvas, DrawParam::default());
        canvas.draw(&Text::new(format!("FPS: {:.0}", ctx.time.fps())), DrawParam::default().color(Color::BLACK));

        canvas.finish(ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.settings.sc_width = width;
        self.settings.sc_height = height;
        Ok(())
    }
}

fn main() -> GameResult {
    let sc_width: f32 = 1280.0;
    let sc_height: f32 = 960.0;

    let (mut ctx, event_loop) = ContextBuilder::new("advent", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Advent"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(sc_width, sc_height))
        .build()
        .unwrap();
    ctx.fs.mount(&PathBuf::from("./resources"), true);

    defs::load_all_mods();

    let texture_paths = defs::get_paths();
    let atlas = Atlas::new(&ctx, &texture_paths)?;

    let world = world::World::new(100, 60, 64);

    let mut settings = Settings::new();
    settings.aspect = Vec2::splat(world.tile_size as f32 / 16.0);
    settings.sc_width = sc_width;
    settings.sc_height = sc_height;

    let game = Game::new(&mut ctx, settings, atlas, world)?;

    event::run(ctx, event_loop, game);
}
