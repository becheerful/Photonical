mod defs;
mod ecs;
mod energy;
mod game;
mod player;
mod res;
mod scripts;
mod settings;
mod states;
mod ui;
mod world;

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
    let settings = settings::Settings::new(sc_width, sc_height);

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

    let game = game::GameHandler::new(&mut ctx, atlas, script_engine, settings)?;

    ggez::event::run(ctx, event_loop, game);
}
