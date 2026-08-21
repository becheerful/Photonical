mod defs;
mod ecs;
mod game;
mod json;
mod network;
mod player;
mod res;
mod scripts;
mod states;
mod ui;
mod world;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub screen_width: f32,
    pub screen_height: f32,
    pub fullscreen_type: ggez::conf::FullscreenType,
    pub font: String,
    pub ui_mouse_wheel_sensitivity: f32,
    pub aspect_mouse_wheel_sensitivity: f32,
    pub show_fps: bool,
}

fn load_settings() -> ggez::GameResult<Settings> {
    let content = std::fs::read_to_string("./settings.toml")
        .map_err(|e| ggez::GameError::FilesystemError(e.to_string()))?;
    toml::from_str(&content).map_err(|e| ggez::GameError::CustomError(e.to_string()))
}

fn main() -> ggez::GameResult {
    let settings = load_settings()?;

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("photonical", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Photonical"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(settings.screen_width, settings.screen_height)
                .fullscreen_type(settings.fullscreen_type)
                .resizable(true),
        )
        .build()?;

    ctx.fs.mount(&std::path::Path::new("."), true);
    ctx.gfx.add_font(
        "ScienceGothic",
        ggez::graphics::FontData::from_path(&ctx.fs, &settings.font)?,
    );

    // ctx.gfx.set_window_icon(&ctx.fs, "/assets/textures/blocks/collimator.png")?;
    ggez::input::mouse::set_cursor_type(&mut ctx, ggez::input::mouse::CursorIcon::Crosshair);

    let mut reg = defs::Registry::new()?;

    let mut script_engine = scripts::ScriptEngine::new();
    defs::link_scripts(&reg, &mut script_engine);
    script_engine
        .init_api()
        .expect("Error during Lua API initialization");

    let mut paths_list = defs::get_paths(&reg);
    for path in ui::PlayerUI::collect_ui_paths() {
        paths_list.insert(path);
    }

    let atlas = res::Atlas::new(&ctx, &paths_list)?;

    defs::gen_uv_cache(&mut reg, &atlas)?;

    defs::REGISTRY
        .set(reg)
        .expect("Game registry already initialized");

    let game = game::GameHandler::new(&mut ctx, atlas, script_engine, settings)?;

    ggez::event::run(ctx, event_loop, game);
}
