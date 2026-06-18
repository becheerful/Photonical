use std::{cell::RefCell, sync::Arc};

use ggez::{conf::FullscreenType, glam::Vec2};

mod game;
mod world;
mod res;
mod player;
mod defs;
mod scripts;
mod energy;

const MISSING_TEX: &str = "./resources/assets/textures/missing.png";
const TEXTURE_SIZE: f32 = 16.0;

const PARAM_BLOCK_INDEX_IN_REGISTRY: &str = "raw_id";
const PARAM_ENTITY_ID: &str = "entity_id";
const PARAM_NETWORK_ID: &str = "net_id";
const PARAM_POSITION: &str = "pos";
const PARAM_ENERGY_POWER: &str = "power";
const PARAM_ENERGY_DEMAND: &str = "demand";
const PARAM_ENERGY_MASK: &str = "mask";

const NETWORK_MASK_PRODUCER: u8 = 1;
const NETWORK_MASK_CONSUMER: u8 = 2;
const NETWORK_MASK_STORAGE: u8 = 3;


pub type WorldRef = Arc<RefCell<world::World>>;

struct Settings {
    pub aspect: Vec2,
    pub sc_width: f32,
    pub sc_height: f32,
    pub fullscreen_type: FullscreenType,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            aspect: Vec2::ONE,
            sc_width: 640.0,
            sc_height: 480.0,
            fullscreen_type: FullscreenType::Windowed
        }
    }
}

fn main() -> ggez::GameResult {
    let sc_width: f32 = 1920.0;
    let sc_height: f32 = 1080.0;

    let (ctx, event_loop) = ggez::ContextBuilder::new("photonical", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Photonical"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(sc_width, sc_height).resizable(true))
        .build()?;
    ctx.fs.mount(&std::path::PathBuf::from("./resources"), true);

    let mut reg = defs::Registry::new();

    let atlas = res::Atlas::new(&ctx, &defs::get_paths(&reg))?;
    defs::gen_uv_cache(&mut reg, &atlas);

    let mut script_engine = scripts::ScriptEngine::new();
    defs::link_scripts(&mut reg, &mut script_engine);

    defs::REGISTRY.set(reg).expect("Game registry already initialized");

    let world_ref = Arc::new(RefCell::new(world::World::new(128, 64, 64.0)));
    let world = world_ref.borrow();

    script_engine.init_api(world_ref.clone()).expect("Error during Lua API initialization");

    let mut settings = Settings::new();
    settings.aspect = Vec2::splat(world.map.tile_size / TEXTURE_SIZE);
    settings.sc_width = sc_width;
    settings.sc_height = sc_height;

    let camera = player::Camera::new(&world.map, &settings);

    drop(world);
    let game = game::Game::new(atlas, world_ref, camera, script_engine, settings);

    ggez::event::run(ctx, event_loop, game);
}
