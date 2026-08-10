mod defs;
mod ecs;
mod energy;
mod game;
mod player;
mod res;
mod scripts;
mod states;
mod ui;
mod world;

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
    pub fn new(sc_width: f32, sc_height: f32) -> Self {
        Settings {
            sc_width,
            sc_height,
            fullscreen_type: ggez::conf::FullscreenType::Windowed,
            mouse_wheel_sensitivity: 0.5,
        }
    }
}

fn main() -> ggez::GameResult {
    let sc_width: f32 = 640.0;
    let sc_height: f32 = 480.0;

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("photonical", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Photonical"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(sc_width, sc_height)
                .resizable(true),
        )
        .build()?;
    ctx.fs.mount(&std::path::PathBuf::from("./resources"), true);
    // ctx.gfx.set_window_icon(&ctx.fs, "/assets/textures/blocks/collimator.png")?;
    ggez::input::mouse::set_cursor_type(&mut ctx, ggez::input::mouse::CursorIcon::Crosshair);

    let mut reg = defs::Registry::new()?;
    let settings = Settings::new(sc_width, sc_height);

    let mut script_engine = scripts::ScriptEngine::new();
    defs::link_scripts(&reg, &mut script_engine);
    script_engine
        .init_api()
        .expect("Error during Lua API initialization");

    let mut paths_list = defs::get_paths(&reg);
    for path in crate::ui::PlayerUI::collect_ui_paths() {
        paths_list.insert(path);
    }

    let atlas = res::Atlas::new(&ctx, &paths_list)?;

    defs::gen_uv_cache(&mut reg, &atlas)?;

    defs::REGISTRY
        .set(reg)
        .expect("Game registry already initialized");

    let game = game::GameHandler::new(&mut ctx, atlas, script_engine, settings);

    ggez::event::run(ctx, event_loop, game);
}
