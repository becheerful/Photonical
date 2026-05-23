use std::path::PathBuf;

use ggez::{conf::FullscreenType, glam::Vec2};

mod game;
mod world;
mod res;
mod player;
mod defs;
mod scripts;

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

fn main() -> ggez::GameResult {
    let sc_width: f32 = 1920.0;
    let sc_height: f32 = 1080.0;

    let (ctx, event_loop) = ggez::ContextBuilder::new("photonical", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Photonical"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(sc_width, sc_height).resizable(true))
        .build()?;
    ctx.fs.mount(&PathBuf::from("./resources"), true);

    let mut reg = defs::Registry::new();

    let atlas = res::Atlas::new(&ctx, &defs::get_paths(&reg))?;
    defs::gen_uv_cache(&mut reg, &atlas);

    let mut script_engine = scripts::ScriptEngine::new();
    defs::link_scripts(&mut reg, &mut script_engine);

    if let Err(_) = defs::REGISTRY.set(reg) {
        eprintln!("Game registry already initialized")
    }

    if let Err(_) = script_engine.init_api() {
        eprintln!("Error during Lua API initialization");
    }

    // the width and height must be divisable by 16
    let world = world::World::new(128, 64, 64.0);

    let mut settings = Settings::new();
    settings.aspect = Vec2::splat(world.tile_size / TEXTURE_SIZE);
    settings.sc_width = sc_width;
    settings.sc_height = sc_height;

    let camera = player::Camera::new(&world, &settings);

    let game = game::Game::new(atlas, world, camera, script_engine, settings);

    ggez::event::run(ctx, event_loop, game);
}
