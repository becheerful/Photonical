use ggez::{
    Context,
    GameError,
    GameResult,
    conf::FullscreenType,
    event::MouseButton,
    glam::UVec2,
    graphics::DrawParam,
    input::keyboard::KeyCode
};

use crate::{
    WorldRef,
    defs::registry,
    game::SharedData,
    player::Player,
    scripts::ScriptEngine,
    world::{BlockType, NetworkId, Position, Table}
};


pub struct PlayingState {
    pub world: WorldRef,
    pub player: Player,
    pub cur_block: Option<u32>,
    pub cur_net: Option<u32>,
}

impl PlayingState {
    pub fn new(world: WorldRef, player: Player) -> Self {
        Self {
            world,
            player,
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

    pub fn handle_click_on_block(&mut self, x: u16, y: u16, dt: f32, script_engine: &mut ScriptEngine) -> GameResult {
        let mut world = self.world.borrow_mut();
        let index = world.map.index(x, y);

        if world.map.block_entities[index].is_none() {
            if self.cur_block.is_none() {
                return Ok(());
            }

            let cur_block = self.cur_block.unwrap();

            let bd = registry().get_block_directly(cur_block).unwrap();
            let has_network = !bd.net.is_empty();

            if has_network && let Some(net_mask) = bd.net.get(crate::PARAM_ENERGY_MASK) {
                let mask = net_mask.as_u64().expect("") as u8;
                let net_id = self.cur_net.unwrap_or(world.energy_master.networks.len() as u32);

                match mask {
                    crate::NETWORK_MASK_PRODUCER => {
                        let power = bd.net.get(crate::PARAM_ENERGY_POWER).ok_or(
                            GameError::ConfigError("Missing parameter `power` for network mask 1".to_owned())
                        )?.as_i64().expect("");

                        if !world.check_for_space(x, y, bd.size) {
                            return Ok(());
                        }

                        let e = Some(world.ecs.spawn((
                            BlockType(cur_block),
                            Position(UVec2::new(x as u32, y as u32)),
                            crate::world::PowerProducer(power as u32),
                            NetworkId(net_id),
                        )));

                        world.place_block(x, y, bd.size, e);
                        world.energy_master.add_producer(net_id, power);
                    }

                    crate::NETWORK_MASK_CONSUMER => {
                        let demand = bd.net.get(crate::PARAM_ENERGY_DEMAND).ok_or(
                            GameError::ConfigError("Missing parameter `demand` for network mask 2".to_owned())
                        )?.as_i64().expect("");

                        if !world.check_for_space(x, y, bd.size) {
                            return Ok(());
                        }

                        let e = Some(world.ecs.spawn((
                            BlockType(cur_block),
                            Position(UVec2::new(x as u32, y as u32)),
                            crate::world::PowerConsumer(demand as u32),
                            NetworkId(net_id),
                        )));

                        world.place_block(x, y, bd.size, e);
                        world.energy_master.add_consumer(net_id, demand);
                    }

                    crate::NETWORK_MASK_STORAGE => {
                        if !world.check_for_space(x, y, bd.size) {
                            return Ok(());
                        }

                        let e = Some(world.ecs.spawn((
                            BlockType(cur_block),
                            Position(UVec2::new(x as u32, y as u32)),
                            NetworkId(net_id),
                        )));

                        world.place_block(x, y, bd.size, e);
                        world.energy_master.add_storage(net_id);
                    }

                    _ => {
                        return Err(GameError::ConfigError("No such mask".to_owned()));
                    }
                }

                self.cur_net = None;
            }

            if bd.script.is_some() {
                if world.map.block_entities[index].is_none() {
                    let e = Some(world.ecs.spawn((
                        BlockType(cur_block),
                        Position(UVec2::new(x as u32, y as u32)),
                        Table(None),
                    )));

                    world.place_block(x, y, bd.size, e);
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
            if let Err(e) = script_engine.run_lua_function(
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
}

impl crate::game::State for PlayingState {
    fn update(&mut self, data: &mut SharedData, ctx: &mut Context) -> GameResult {
        if let Err(e) = data.script_engine.update(&self.world.borrow(), ctx.time.delta().as_secs_f32()) {
            eprintln!("{e}");
        }

        Ok(())
    }

    fn draw(&self, data: &SharedData, ctx: &mut Context) -> GameResult {
        let mut world = self.world.borrow_mut();
        let tile_size = world.map.tile_size;
        let aspect = world.aspect;

        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::WHITE);
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let mut array = ggez::graphics::InstanceArray::new(ctx, data.atlas.image.clone());

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

        ggez::graphics::Drawable::draw(&array, &mut canvas, DrawParam::default());
        canvas.draw(
            &ggez::graphics::Text::new(format!("FPS: {:.0}", ctx.time.fps())),
            DrawParam::default().color(ggez::graphics::Color::RED)
        );

        // should be last because of scissors
        self.player.draw(&mut canvas, &data.atlas)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn window_resize(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        new_width: f32,
        new_height: f32
    ) -> GameResult {
        self.player.ui.resize_event(new_width, new_height);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let Some(keycode) = input.keycode {
            match keycode {
                KeyCode::F11 => {
                    data.settings.fullscreen_type = match data.settings.fullscreen_type {
                        FullscreenType::Windowed => FullscreenType::Desktop,
                        _ => FullscreenType::Windowed,
                    };

                    ctx.gfx.set_fullscreen(data.settings.fullscreen_type)?;
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

    fn mouse_button_down_event(&mut self, data: &mut SharedData, ctx: &mut Context, button: MouseButton, mx: f32, my: f32) -> GameResult {
        match button {
            MouseButton::Left => {
                let index = self.player.ui.block_list.mouse_button_down_event(
                    &data.settings,
                    ggez::glam::Vec2::new(mx, my)
                );

                if index != None {
                    self.cur_block = index;
                    return Ok(());
                }

                let (x, y) = self.point_to_block_pos(mx, my);
                if ctx.keyboard.is_key_pressed(KeyCode::RShift) || ctx.keyboard.is_key_pressed(KeyCode::LShift) {
                    self.cur_net = self.get_block_network_id(x, y);
                } else {
                    self.handle_click_on_block(x, y, ctx.time.delta().as_secs_f32(), &mut data.script_engine)?;
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

    fn mouse_button_up_event(&mut self, data: &mut SharedData, ctx: &mut Context, button: MouseButton, mx: f32, my: f32) -> GameResult {
        if button == MouseButton::Left {
            let (x, y) = self.point_to_block_pos(mx, my);
            let mut world = self.world.borrow_mut();
            let index = world.map.index(x, y);

            if let Err(e) = data.script_engine.run_lua_function(
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

    fn mouse_wheel_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        _x: f32,
        y: f32
    ) -> GameResult {
        if !self.player.ui.block_list.scroll_event(&data.settings, ctx.mouse.position(), y) {
            let mut world = self.world.borrow_mut();
            world.aspect += y * 0.1;
            world.map.tile_size = crate::TEXTURE_SIZE * world.aspect.x;
        }

        Ok(())
    }
}
