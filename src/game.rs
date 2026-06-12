use ggez::{
    Context,
    GameResult,
    conf::FullscreenType,
    event::EventHandler,
    glam::UVec2,
    graphics::{Color, DrawParam, Drawable},
    input::keyboard::KeyCode,
};

use crate::{
    NETWORK_MASK_CONSUMER,
    NETWORK_MASK_STORAGE,
    PARAM_ENERGY_DEMAND,
    PARAM_ENERGY_MASK,
    PARAM_ENERGY_POWER,
    Settings,
    WorldRef,
    defs::registry,
    player::Camera,
    res::Atlas,
    scripts::ScriptEngine,
    world::{BlockType, Position, PowerConsumer, PowerProducer, Table}
};

pub struct Game {
    pub atlas: Atlas,
    pub world: WorldRef,
    pub camera: Camera,
    pub script_engine: ScriptEngine,
    pub settings: Settings,
    pub cur_block: u32,
}

impl Game {
    pub fn new(atlas: Atlas, world: WorldRef, camera: Camera, script_engine: ScriptEngine, settings: Settings) -> Self {
        Game {
            atlas,
            world,
            camera,
            script_engine,
            settings,
            cur_block: registry().get_block_index("photonical:collimator").expect("Block not found"),
        }
    }

    fn update_game(&mut self, dt: f32) {
        if let Err(e) = self.script_engine.update(&self.world.borrow(), dt) {
            eprintln!("{}", e);
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
                KeyCode::Key1 => self.cur_block = 0,
                KeyCode::Key2 => self.cur_block = 1,
                KeyCode::Key3 => self.cur_block = 2,
                KeyCode::Key4 => self.cur_block = 3,
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
            let mut world = self.world.borrow_mut();

            let mouse_point = ctx.mouse.position();
            let rel_x = mouse_point.x + self.camera.pos.x;
            let rel_y = mouse_point.y + self.camera.pos.y;

            let tile_x = (rel_x / world.tile_size) as u16;
            let tile_y = (rel_y / world.tile_size) as u16;

            let index = world.index(tile_x, tile_y);

            if world.get(tile_x, tile_y).is_none() {
                let bd = registry().get_block_directly(self.cur_block).unwrap();
                let has_network = !bd.net.is_empty();

                if has_network {
                    if let Some(net_mask) = bd.net.get(PARAM_ENERGY_MASK) {
                        let mask = net_mask.as_u64().expect("") as u8;
                        if mask & NETWORK_MASK_STORAGE == NETWORK_MASK_STORAGE {
                            world.block_entities[index] = Some(world.ecs.spawn((
                                BlockType(self.cur_block),
                                Position(UVec2::new(tile_x as u32, tile_y as u32)),
                            )));
                        } else if mask & NETWORK_MASK_CONSUMER == NETWORK_MASK_CONSUMER {
                            let e = Some(world.ecs.spawn((
                                BlockType(self.cur_block),
                                Position(UVec2::new(tile_x as u32, tile_y as u32)),
                                PowerConsumer(bd.net.get(PARAM_ENERGY_DEMAND).unwrap().as_i64().expect("") as u32),
                            )));

                            world.block_entities[index] = e;
                            world.energy_master.add_consumer(e.unwrap());
                        } else {
                            let e = Some(world.ecs.spawn((
                                BlockType(self.cur_block),
                                Position(UVec2::new(tile_x as u32, tile_y as u32)),
                                PowerProducer(bd.net.get(PARAM_ENERGY_POWER).unwrap().as_i64().expect("") as u32),
                            )));

                            world.block_entities[index] = e;
                            world.energy_master.add_producer(e.unwrap());
                        }

                        world.update_networks();
                    }
                }

                if bd.script.is_some() {
                    if world.get_directly(index).is_none() {
                        world.block_entities[index] = Some(world.ecs.spawn((
                            BlockType(self.cur_block),
                            Position(UVec2::new(tile_x as u32, tile_y as u32)),
                            Table(None),
                        )));
                    } else {
                        let e = world.block_entities[index].unwrap();
                        if let Err(e) = world.ecs.insert_one(e, Table(None)) {
                            eprintln!("{}", e);
                        }
                    }
                } else if !has_network {
                    world.static_tiles[index] = (self.cur_block, UVec2::new(tile_x as u32, tile_y as u32));
                }
            }
        } else if ctx.mouse.button_pressed(ggez::event::MouseButton::Right) {
            let mut world = self.world.borrow_mut();

            let mouse_point = ctx.mouse.position();
            let rel_x = mouse_point.x + self.camera.pos.x;
            let rel_y = mouse_point.y + self.camera.pos.y;

            let tile_x = (rel_x / world.tile_size) as u16;
            let tile_y = (rel_y / world.tile_size) as u16;

            let index = world.index(tile_x, tile_y);

            if world.get(tile_x, tile_y).is_some() {
                if let Some(entity) = world.block_entities[index] {
                    if let Err(e) = world.ecs.despawn(entity) {
                        eprintln!("{}", e);
                    }

                    world.block_entities[index] = None;
                }
            }
        }

        self.update_game(ctx.time.delta().as_secs_f32());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let world = self.world.borrow();

        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, Color::WHITE);
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let mut array = ggez::graphics::InstanceArray::new(ctx, self.atlas.image.clone());

        for (id, pos) in world.static_tiles.iter() {
            array.push(DrawParam::default()
                .src(registry().get_block_directly(*id).unwrap().uv.unwrap())
                .dest(pos.as_vec2() * world.tile_size - self.camera.pos)
                .scale(self.settings.aspect)
            );
        }

        for (_, (id, pos)) in world.ecs.query::<(&BlockType, &Position)>().iter() {
            array.push(DrawParam::default()
                .src(registry().get_block_directly(id.0).unwrap().uv.unwrap())
                .dest(pos.0.as_vec2() * world.tile_size - self.camera.pos)
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
