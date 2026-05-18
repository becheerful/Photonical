use std::{cell::{Ref, RefCell}, path::PathBuf, rc::Rc};

use ggez::{
    Context, ContextBuilder, GameResult,
    conf::FullscreenType,
    event::EventHandler,
    glam::Vec2,
    graphics::{Color, DrawParam, Drawable, InstanceArray, Sampler, Text},
    input::keyboard::{KeyCode, KeyInput}
};

use crate::{defs::{Registry, registry}, player::Camera, res::Atlas, script::{ScriptEngine, WorldRef}, world::World};

mod world;
mod res;
mod player;
mod defs;
mod script;

const MISSING_TEX: &str = "./resources/assets/textures/missing.png";
const TEXTURE_SIZE: f32 = 16.0;

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
    pub world_ref: WorldRef,
    pub camera: Camera,
    pub script_engine: ScriptEngine,
    pub settings: Settings,
}

impl Game {
    fn new(atlas: Atlas, world_ref: WorldRef, camera: Camera, script_engine: ScriptEngine, settings: Settings) -> Self {
        Game {
            atlas,
            world_ref,
            camera,
            script_engine,
            settings,
        }
    }

    fn update_game(&mut self, dt: f32) {
        if let Err(e) = self.script_engine.update(self.world_ref.clone(), dt) {
            eprintln!("{}", e)
        }
    }

    pub fn world(&self) -> Ref<'_, World> {
        self.world_ref.borrow()
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
                        let bounds = self.world().bounds;
                        self.camera.move_towards(direction, bounds);
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

            let tile_x = (rel_x / self.world().tile_size) as usize;
            let tile_y = (rel_y / self.world().tile_size) as usize;
            
            let mut world = self.world_ref.borrow_mut();
            if let Some(block) = world.get_mut(tile_x, tile_y) {
                if block.id == registry().get_block_index("photonical:stone").unwrap() {
                    let block_id = "photonical:collimator";
                    if let Some(index) = registry().get_block_index(block_id) {
                        block.id = index;
                        let width = world.width;
                        world.mechanisms.push(tile_y * width + tile_x);
                    } else {
                        eprintln!("Block '{}' was not found", block_id);
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

        for block in &self.world().map {
            // We can call `.unwrap()` here because a world contains this block.
            // Since the world contains this block, therefore, this block exists in the regisry.
            let def = registry().get_block_directly(block.id).unwrap();
            array.push(DrawParam::default()
                .src(def.uv.unwrap())
                .dest(block.pos * self.world().tile_size - self.camera.pos)
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
        .build()
        .unwrap();
    ctx.fs.mount(&PathBuf::from("./resources"), true);
    
    let mut registry = Registry::new();

    let atlas = Atlas::new(&ctx, &defs::get_paths(&registry))?;
    defs::gen_uv_cache(&mut registry, &atlas);
    
    let mut script_engine = ScriptEngine::new();
    defs::link_scripts(&mut registry, &mut script_engine);

    if let Err(_) = defs::REGISTRY.set(registry) {
        eprintln!("Game registry already initialized")
    }

    let world = Rc::new(RefCell::new(World::new(100, 60, 64.0))); 
    if let Err(e) = script_engine.init_api(world.clone()) {
        eprintln!("{}", e);
    }

    let camera = Camera::new();

    let mut settings = Settings::new();
    settings.aspect = Vec2::splat(world.borrow().tile_size / TEXTURE_SIZE);
    settings.sc_width = sc_width;
    settings.sc_height = sc_height;

    let game = Game::new(atlas, world, camera, script_engine, settings);

    ggez::event::run(ctx, event_loop, game);
}
