use ggez::{
    Context,
    GameResult,
    conf::FullscreenType,
    event::EventHandler,
    graphics::{Color, DrawParam, Drawable},
    input::keyboard::KeyCode
};

use crate::{
    Settings,
    defs::registry,
    player::Camera,
    res::Atlas,
    scripts::ScriptEngine,
    world::{BlockType, World}
};

pub struct Game {
    pub atlas: Atlas,
    pub world: World,
    pub camera: Camera,
    pub script_engine: ScriptEngine,
    pub settings: Settings,
}

impl Game {
    pub fn new(atlas: Atlas, world: World, camera: Camera, script_engine: ScriptEngine, settings: Settings) -> Self {
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
    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeated: bool) -> GameResult {
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

                    if let Err(e) = self.world.ecs.insert_one(entity, crate::world::Scripted) {
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
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let mut array = ggez::graphics::InstanceArray::new(ctx, self.atlas.image.clone());

        for (_, (id, pos)) in self.world.ecs.query::<(&BlockType, &crate::world::Position)>().iter() {
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
        canvas.draw(
            &ggez::graphics::Text::new(format!("FPS: {:.0}", ctx.time.fps())),
            DrawParam::default().color(Color::RED)
        );

        canvas.finish(ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.settings.sc_width = width;
        self.settings.sc_height = height;
        Ok(())
    }
}
