mod res;
mod defs;
mod scripts;
mod energy;
mod game;
mod world;
mod player;
mod ui;
mod states;

const MISSING_TEX: &str = "./resources/assets/textures/missing.png";
const TEXTURE_SIZE: f32 = 32.0;

// field names for scripts
const PARAM_STRING_ID: &str = "str_id";
const PARAM_BLOCK_INDEX_IN_REGISTRY: &str = "raw_id";
const PARAM_ENTITY_ID: &str = "entity_id";
const PARAM_NETWORK_ID: &str = "net_id";
const PARAM_POSITION: &str = "pos";

// parameter names for .json block definitions
const PARAM_ENERGY_POWER: &str = "power";
const PARAM_ENERGY_DEMAND: &str = "demand";
const PARAM_ENERGY_MASK: &str = "mask";

const LUA_FUNCTION_UPDATE: &str = "update";
const LUA_FUNCTION_MOUSE_BUTTON_DOWN: &str = "on_mouse_button_down";
const LUA_FUNCTION_MOUSE_BUTTON_UP: &str = "on_mouse_button_up";

const NETWORK_MASK_PRODUCER: u8 = 1;
const NETWORK_MASK_CONSUMER: u8 = 2;
const NETWORK_MASK_STORAGE: u8 = 3;


struct Settings {
    pub sc_width: f32,
    pub sc_height: f32,
    pub fullscreen_type: ggez::conf::FullscreenType,
    pub mouse_wheel_sensitivity: f32,
}

impl Settings {
    pub fn new(aspect: f32, sc_width: f32, sc_height: f32) -> Self {
        Settings {
            sc_width,
            sc_height,
            fullscreen_type: ggez::conf::FullscreenType::Windowed,
            mouse_wheel_sensitivity: aspect / 2.0,
        }
    }
}

fn main() -> ggez::GameResult {
    let sc_width: f32 = 640.0;
    let sc_height: f32 = 480.0;

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("photonical", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Photonical"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(sc_width, sc_height).resizable(true))
        .build()?;
    ctx.fs.mount(&std::path::PathBuf::from("./resources"), true);
    ggez::input::mouse::set_cursor_type(&mut ctx, ggez::input::mouse::CursorIcon::Crosshair);

    let mut reg = defs::Registry::new()?;

    let world = world::World::new(&reg, 128, 64, 32.0);

    let settings = Settings::new(world.aspect.x, sc_width, sc_height);

    let mut script_engine = scripts::ScriptEngine::new();
    defs::link_scripts(&reg, &mut script_engine);
    script_engine.init_api().expect("Error during Lua API initialization");

    let mut player_ui = crate::ui::PlayerUI::new(&reg, &world.aspect, &settings);
    let mut paths_list = player_ui.collect_ui_paths();
    paths_list.append(&mut defs::get_paths(&reg));

    let atlas = res::Atlas::new(&ctx, &paths_list)?;
    defs::gen_uv_cache(&mut reg, &atlas)?;

    player_ui.block_list.load_atlas_rect(&atlas)?;
    let player = player::Player::new(&world.map, &settings, player_ui);

    defs::REGISTRY.set(reg).expect("Game registry already initialized");

    let game = game::GameHandler::new(atlas, world, player, script_engine, settings);

    ggez::event::run(ctx, event_loop, game);
}
