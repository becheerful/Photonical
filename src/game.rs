use ggez::{
    Context,
    GameError,
    GameResult,
    conf::FullscreenType,
    event::{EventHandler, MouseButton},
    glam::UVec2,
    graphics::{DrawParam, Drawable},
    input::keyboard::KeyCode,
};

use crate::{
    NETWORK_MASK_CONSUMER,
    NETWORK_MASK_PRODUCER,
    NETWORK_MASK_STORAGE,
    PARAM_ENERGY_CAPACITY,
    PARAM_ENERGY_DEMAND,
    PARAM_ENERGY_MASK,
    PARAM_ENERGY_POWER,
    Settings,
    WorldRef,
    defs::registry,
    player::Player,
    res::Atlas,
    scripts::ScriptEngine,
    world::{BlockType, NetworkId, Position, PowerConsumer, PowerProducer, PowerStorage, Table}
};

pub struct Game {
    pub atlas: Atlas,
    pub world: WorldRef,
    pub player: Player,
    pub script_engine: ScriptEngine,
    pub settings: Settings,
    pub cur_block: Option<u32>,
    pub cur_net: Option<u32>,
}

impl Game {
    pub fn new(atlas: Atlas, world: WorldRef, player: Player, script_engine: ScriptEngine, settings: Settings) -> Self {
        Game {
            atlas,
            world,
            player,
            script_engine,
            settings,
            cur_block: None,
            cur_net: Some(0),
        }
    }

    pub fn point_to_block_pos(&self, px: f32, py: f32) -> (u16, u16) {
        let world = self.world.borrow();
        (
            ((px + self.player.camera.pos.x) / world.map.tile_size) as u16,
            ((py + self.player.camera.pos.y) / world.map.tile_size) as u16,
        )
    }

    pub fn handle_click_on_block(&mut self, dt: f32, x: u16, y: u16) -> GameResult {
        let mut world = self.world.borrow_mut();
        let index = world.map.index(x, y);

        if world.map.block_entities[index].is_none() {
            if self.cur_block.is_none() {
                return Ok(());
            }

            let cur_block = self.cur_block.unwrap();

            let bd = registry().get_block_directly(cur_block).unwrap();
            let has_network = !bd.net.is_empty();

            if has_network {
                if let Some(net_mask) = bd.net.get(PARAM_ENERGY_MASK) {
                    let mask = net_mask.as_u64().expect("") as u8;
                    let net_id = self.cur_net.unwrap_or(world.energy_master.networks.len() as u32);

                    match mask {
                        NETWORK_MASK_PRODUCER => {
                            let power = bd.net.get(PARAM_ENERGY_POWER).ok_or(
                                GameError::ConfigError("Missing parameter `power` for network mask 1".to_owned())
                            )?.as_i64().expect("");

                            let e = Some(world.ecs.spawn((
                                BlockType(cur_block),
                                Position(UVec2::new(x as u32, y as u32)),
                                PowerProducer(power as u32),
                                NetworkId(net_id),
                            )));

                            for col in x..(x + bd.size) {
                                for row in y..(y + bd.size) {
                                    let index = world.map.index(col, row);
                                    world.map.block_entities[index] = e;
                                }
                            }

                            world.energy_master.add_producer(net_id, power);
                        }

                        NETWORK_MASK_CONSUMER => {
                            let demand = bd.net.get(PARAM_ENERGY_DEMAND).ok_or(
                                GameError::ConfigError("Missing parameter `demand` for network mask 2".to_owned())
                            )?.as_i64().expect("");

                            let e = Some(world.ecs.spawn((
                                BlockType(cur_block),
                                Position(UVec2::new(x as u32, y as u32)),
                                PowerConsumer(demand as u32),
                                NetworkId(net_id),
                            )));

                            for col in x..(x + bd.size) {
                                for row in y..(y + bd.size) {
                                    let index = world.map.index(col, row);
                                    world.map.block_entities[index] = e;
                                }
                            }

                            world.energy_master.add_consumer(net_id, demand);
                        }

                        NETWORK_MASK_STORAGE => {
                            let capacity = bd.net.get(PARAM_ENERGY_CAPACITY).ok_or(
                                GameError::ConfigError("Missing parameter `capacity` for network mask 3".to_owned())
                            )?.as_i64().expect("");

                            let e = Some(world.ecs.spawn((
                                BlockType(cur_block),
                                Position(UVec2::new(x as u32, y as u32)),
                                PowerStorage(0, capacity as u32),
                                NetworkId(net_id),
                            )));

                            for col in x..(x + bd.size) {
                                for row in y..(y + bd.size) {
                                    let index = world.map.index(col, row);
                                    world.map.block_entities[index] = e;
                                }
                            }

                            world.energy_master.add_storage(net_id);
                        }

                        _ => {
                            return Err(GameError::ConfigError("No such mask".to_owned()));
                        }
                    }

                    self.cur_net = None;
                }
            }

            if bd.script.is_some() {
                if world.map.block_entities[index].is_none() {
                    let e = Some(world.ecs.spawn((
                        BlockType(cur_block),
                        Position(UVec2::new(x as u32, y as u32)),
                        Table(None),
                    )));

                    for col in x..(x + bd.size) {
                        for row in y..(y + bd.size) {
                            let index = world.map.index(col, row);
                            world.map.block_entities[index] = e;
                        }
                    }
                } else {
                    let e = world.map.block_entities[index].unwrap();
                    if let Err(e) = world.ecs.insert_one(e, Table(None)) {
                        eprintln!("{e}");
                    }
                }
            } else if !has_network {
                world.map.static_tiles[index] = (cur_block, UVec2::new(x as u32, y as u32));
            }

            self.cur_block = None;
        } else {
            drop(world);
            let world = self.world.borrow();
            if let Err(e) = self.script_engine.run_lua_function(
                crate::LUA_FUNCTION_MOUSE_BUTTON_DOWN,
                &world,
                index,
                dt,
            ) {
                eprintln!("{e}");
            }
        }

        Ok(())
    }

    pub fn remove_block(&mut self, x: u16, y: u16) {
        let mut world = self.world.borrow_mut();
        world.remove_entity(x, y);
    }

    pub fn get_block_network_id(&self, x: u16, y: u16) -> Option<u32> {
        let world = self.world.borrow();
        let index = world.map.index(x, y);

        if let Some(entity) = world.map.block_entities[index] {
            if let Ok(id) = world.ecs.get::<&NetworkId>(entity) {
                return Some(id.0);
            }
        }

        None
    }

    fn update_game(&mut self, dt: f32) {
        let mut world = self.world.borrow_mut();
        world.update();

        if let Err(e) = self.script_engine.update(&mut world, dt) {
            eprintln!("{e}");
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
                    if let Some(direction) = self.player.camera.get_movement_vector(key) {
                        self.player.camera.move_towards(direction);
                    }
                }
            }
        }

        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: ggez::event::MouseButton, mx: f32, my: f32) -> GameResult {
        match button {
            MouseButton::Left => {
                let index = self.player.ui.block_list.mouse_button_down_event(&self.settings, ggez::glam::Vec2::new(mx, my));
                if index != None {
                    self.cur_block = index;
                    return Ok(());
                }

                let (x, y) = self.point_to_block_pos(mx, my);
                if ctx.keyboard.is_key_pressed(KeyCode::RShift) || ctx.keyboard.is_key_pressed(KeyCode::LShift) {
                    self.cur_net = self.get_block_network_id(x, y);
                } else {
                    self.handle_click_on_block(ctx.time.delta().as_secs_f32(), x, y)?;
                }
            }

            MouseButton::Middle => {
                let (x, y) = self.point_to_block_pos(mx, my);
                let mut world = self.world.borrow_mut();

                if let Some(entity) = world.map.get(x, y) {
                    if let Ok((id, network)) = world.ecs.query_one_mut::<(
                        &BlockType, Option<&NetworkId>
                    )>(entity) {
                        self.cur_block = Some(id.0);
                        if let Some(net_id) = network {
                            self.cur_net = Some(net_id.0)
                        }
                    }
                }
            }

            MouseButton::Right => {
                let (x, y) = self.point_to_block_pos(mx, my);
                self.remove_block(x, y);
            }

            MouseButton::Other(_) => {}
        }

        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, mx: f32, my: f32) -> GameResult {
        if button == MouseButton::Left {
            let (x, y) = self.point_to_block_pos(mx, my);
            let mut world = self.world.borrow_mut();
            let index = world.map.index(x, y);

            if let Err(e) = self.script_engine.run_lua_function(
                crate::LUA_FUNCTION_MOUSE_BUTTON_UP,
                &mut world,
                index,
                ctx.time.delta().as_secs_f32(),
            ) {
                eprintln!("{e}")
            }
        }

        Ok(())
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) -> GameResult {
        if !self.player.ui.block_list.scroll_event(&self.settings, ctx.mouse.position(), y) {
            let mut world = self.world.borrow_mut();
            world.aspect += y * 0.1;
            world.map.tile_size = crate::TEXTURE_SIZE * world.aspect.x;
        }

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.update_game(ctx.time.delta().as_secs_f32());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut world = self.world.borrow_mut();
        let tile_size = world.map.tile_size;
        let aspect = world.aspect;

        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::WHITE);
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let mut array = ggez::graphics::InstanceArray::new(ctx, self.atlas.image.clone());

        for (id, pos) in world.map.static_tiles.iter() {
            array.push(DrawParam::default()
                .src(registry().get_block_directly(*id).unwrap().uv.unwrap())
                .dest(pos.as_vec2() * tile_size - self.player.camera.pos)
                .scale(aspect)
            );
        }

        for (_, (id, pos)) in world.ecs.query_mut::<(&BlockType, &Position)>() {
            array.push(DrawParam::default()
                .src(registry().get_block_directly(id.0).unwrap().uv.unwrap())
                .dest(pos.0.as_vec2() * tile_size - self.player.camera.pos)
                .scale(aspect)
            );
        }

        array.draw(&mut canvas, DrawParam::default());
        self.player.draw(&mut canvas, &self.atlas)?;

        canvas.draw(
            &ggez::graphics::Text::new(format!("FPS: {:.0}", ctx.time.fps())),
            DrawParam::default().color(ggez::graphics::Color::RED)
        );

        canvas.finish(ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.settings.sc_width = width;
        self.settings.sc_height = height;
        self.player.ui.resize_event(width, height);
        Ok(())
    }
}
