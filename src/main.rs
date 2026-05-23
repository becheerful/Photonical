use std::path::PathBuf;

use ggez::{
    Context, ContextBuilder, GameResult,
    conf::FullscreenType,
    event::EventHandler,
    glam::Vec2,
    graphics::{Color, DrawParam, Drawable, InstanceArray, Sampler, Text},
    input::keyboard::{KeyCode, KeyInput}
};

use crate::{
    defs::{Registry, registry},
    player::Camera,
    res::Atlas,
    script::ScriptEngine,
    world::{BlockType, Position, Scripted, World}
};

mod world;
mod res;
mod player;
mod defs;
mod script;

const MISSING_TEX: &str = "./resources/assets/textures/missing.png";
const TEXTURE_SIZE: f32 = 16.0;
const CHUNK_SIZE: usize = 16;

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
    pub atlas: Atlas,
    pub world: World,
    pub camera: Camera,
    pub script_engine: ScriptEngine,
    pub settings: Settings,
}

impl Game {
    fn new(atlas: Atlas, world: World, camera: Camera, script_engine: ScriptEngine, settings: Settings) -> Self {
        Game {
            atlas,
            world,
            camera,
            script_engine,
            settings,
        }
    }

    fn update_game(&mut self, dt: f32) {
        if let Err(e) = self.script_engine.update(&mut self.world, dt) {
            eprintln!("main:57 {}", e);
        }
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
                    if let Some(direction) = self.camera.get_movement_vector(key) {
                        self.camera.move_towards(direction);
                    }
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.mouse.button_pressed(ggez::event::MouseButton::Left) {
            let mouse_point = ctx.mouse.position();
            let rel_x = mouse_point.x + self.camera.pos.x;
            let rel_y = mouse_point.y + self.camera.pos.y;

            let tile_x = (rel_x / self.world.tile_size) as u16;
            let tile_y = (rel_y / self.world.tile_size) as u16;

            if let Some(entity) = self.world.get(tile_x, tile_y) {
                if self.world.ecs.get::<&BlockType>(entity).expect("Block entity not found").0 == registry().get_block_index("photonical:stone").unwrap() {
                    if let Err(e) = self.world.ecs.remove_one::<BlockType>(entity) {
                        eprintln!("{}", e);
                    }

                    if let Err(e) = self.world.ecs.insert_one(entity, BlockType(registry().get_block_index("photonical:collimator").unwrap())) {
                        eprintln!("{}", e);
                    }

                    if let Err(e) = self.world.ecs.insert_one(entity, Scripted) {
                        eprintln!("{}", e);
                    }
                }
            } else {
                eprintln!("({}, {}) is an invalid position", tile_x, tile_y);
            }
        }

        self.update_game(ctx.time.delta().as_secs_f32());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, Color::WHITE);
        canvas.set_sampler(Sampler::nearest_clamp());

        let mut array = InstanceArray::new(ctx, self.atlas.image.clone());

        for (_, (id, pos)) in self.world.ecs.query::<(&BlockType, &Position)>().iter() {
            // We can call `.unwrap()` here because a world contains this block.
            // Since the world contains this block, therefore, this block exists in the regisry.
            let def = registry().get_block_directly(id.0).unwrap();
            array.push(DrawParam::default()
                .src(def.uv.unwrap())
                .dest(pos.0.as_vec2() * self.world.tile_size - self.camera.pos)
                .scale(self.settings.aspect)
            );
        }

        array.draw(&mut canvas, DrawParam::default());
        canvas.draw(&Text::new(format!("FPS: {:.0}", ctx.time.fps())), DrawParam::default().color(Color::RED));

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
    let sc_width: f32 = 1920.0;
    let sc_height: f32 = 1080.0;

    let (ctx, event_loop) = ContextBuilder::new("photonical", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Photonical"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(sc_width, sc_height).resizable(true))
        .build()?;
    ctx.fs.mount(&PathBuf::from("./resources"), true);

    let mut reg = Registry::new();

    let atlas = Atlas::new(&ctx, &defs::get_paths(&reg))?;
    defs::gen_uv_cache(&mut reg, &atlas);

    let mut script_engine = ScriptEngine::new();
    defs::link_scripts(&mut reg, &mut script_engine);

    if let Err(_) = defs::REGISTRY.set(reg) {
        eprintln!("Game registry already initialized")
    }

    if let Err(_) = script_engine.init_api() {
        eprintln!("Error during Lua API initialization");
    }

    // the width and height must be divisable by 16
    let world = World::new(128, 64, 64.0);

    let mut settings = Settings::new();
    settings.aspect = Vec2::splat(world.tile_size / TEXTURE_SIZE);
    settings.sc_width = sc_width;
    settings.sc_height = sc_height;

    let camera = Camera::new(&world, &settings);

    let game = Game::new(atlas, world, camera, script_engine, settings);

    ggez::event::run(ctx, event_loop, game);
}
