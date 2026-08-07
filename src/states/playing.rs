use ggez::{
    Context, GameError, GameResult,
    conf::FullscreenType,
    event::MouseButton,
    glam::UVec2,
    graphics::{DrawParam, Drawable},
    input::keyboard::KeyCode,
};

use crate::{
    defs::registry,
    ecs::{BlockType, ECS, NetworkId, Position, Table},
    game::SharedData,
    player::Player,
    world::World,
};

pub struct PlayingState {
    player: Player,
    world: World,
    cur_block: Option<u32>,
    cur_net: Option<u32>,
}

impl PlayingState {
    pub fn new(data: &mut SharedData) -> Self {
        let world = World::new(128, 64, 32.0);
        Self {
            player: Player::new(&world, registry(), &data.atlas, &data.settings),
            world,
            cur_block: None,
            cur_net: Some(0),
        }
    }

    fn point_to_block_pos(&self, px: f32, py: f32) -> (u16, u16) {
        (
            ((px + self.player.camera.pos.x) / self.world.map.tile_size) as u16,
            ((py + self.player.camera.pos.y) / self.world.map.tile_size) as u16,
        )
    }

    fn remove_block(&mut self, ecs: &mut ECS, x: u16, y: u16) {
        self.world.remove_entity(ecs, x, y);
    }

    fn get_block_network_id(&self, ecs: &mut ECS, x: u16, y: u16) -> Option<u32> {
        let index = self.world.map.index(x, y);

        if let Some(entity) = self.world.map.block_entities[index] {
            if let Ok(id) = ecs.get::<&NetworkId>(entity) {
                return Some(id.0);
            }
        }

        None
    }

    fn handle_click_on_block(
        &mut self,
        data: &mut SharedData,
        x: u16,
        y: u16,
        dt: f32,
    ) -> GameResult {
        let index = self.world.map.index(x, y);

        if self.world.map.block_entities[index].is_none() {
            if self.cur_block.is_none() {
                return Ok(());
            }

            let cur_block = self.cur_block.unwrap();

            let bd = registry().get_block_directly(cur_block).unwrap();
            let has_network = !bd.net.is_empty();

            if has_network && let Some(net_mask) = bd.net.get(crate::PARAM_ENERGY_MASK) {
                let mask = net_mask.as_u64().expect("") as u8;
                let net_id = self
                    .cur_net
                    .unwrap_or(self.world.energy_master.networks.len() as u32);

                match mask {
                    crate::NETWORK_MASK_PRODUCER => {
                        let power = bd
                            .net
                            .get(crate::PARAM_ENERGY_POWER)
                            .ok_or(GameError::ConfigError(
                                "Missing parameter `power` for network mask 1".to_owned(),
                            ))?
                            .as_f64()
                            .expect("") as f32;

                        if !self.world.check_for_space(x, y, bd.size) {
                            return Ok(());
                        }

                        let e = Some(data.ecs.spawn((
                            data.atlas.make_texture_rect(
                                &registry().get_block_directly(cur_block).unwrap().texture,
                            )?,
                            BlockType(cur_block),
                            Position(x, y),
                            crate::ecs::PowerProducer(power),
                            NetworkId(net_id),
                        )));

                        self.world.place_block(x, y, bd.size, e);
                        self.world.energy_master.add_producer(net_id, power);
                    }

                    crate::NETWORK_MASK_CONSUMER => {
                        let demand = bd
                            .net
                            .get(crate::PARAM_ENERGY_DEMAND)
                            .ok_or(GameError::ConfigError(
                                "Missing parameter `demand` for network mask 2".to_owned(),
                            ))?
                            .as_f64()
                            .expect("") as f32;

                        if !self.world.check_for_space(x, y, bd.size) {
                            return Ok(());
                        }

                        let e = Some(data.ecs.spawn((
                            data.atlas.make_texture_rect(
                                &registry().get_block_directly(cur_block).unwrap().texture,
                            )?,
                            BlockType(cur_block),
                            Position(x, y),
                            crate::ecs::PowerConsumer(demand),
                            NetworkId(net_id),
                        )));

                        self.world.place_block(x, y, bd.size, e);
                        self.world.energy_master.add_consumer(net_id, demand);
                    }

                    crate::NETWORK_MASK_STORAGE => {
                        if !self.world.check_for_space(x, y, bd.size) {
                            return Ok(());
                        }

                        let e = Some(data.ecs.spawn((
                            data.atlas.make_texture_rect(
                                &registry().get_block_directly(cur_block).unwrap().texture,
                            )?,
                            BlockType(cur_block),
                            Position(x, y),
                            NetworkId(net_id),
                        )));

                        self.world.place_block(x, y, bd.size, e);
                        self.world.energy_master.add_storage(net_id);
                    }

                    _ => {
                        return Err(GameError::ConfigError("No such mask".to_owned()));
                    }
                }

                self.cur_net = None;
            }

            if bd.script.is_some() {
                if self.world.map.block_entities[index].is_none() {
                    let e = Some(data.ecs.spawn((
                        data.atlas.make_texture_rect(
                            &registry().get_block_directly(cur_block).unwrap().texture,
                        )?,
                        BlockType(cur_block),
                        Position(x, y),
                        Table(None),
                    )));

                    self.world.place_block(x, y, bd.size, e);
                } else {
                    let e = self.world.map.block_entities[index].unwrap();
                    if let Err(e) = data.ecs.insert_one(e, Table(None)) {
                        eprintln!("{e}");
                    }
                }
            } else if !has_network {
                self.world.map.static_tiles[index] = (cur_block, UVec2::new(x as u32, y as u32));
            }

            self.cur_block = None;
        } else {
            if let Err(e) = data.script_engine.run_lua_function(
                &mut data.ecs,
                crate::LUA_FUNCTION_MOUSE_BUTTON_DOWN,
                &mut self.world,
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
        self.player.camera.update();

        if let Err(e) = data.script_engine.update(
            &mut data.ecs,
            &mut self.world,
            ctx.time.delta().as_secs_f32(),
        ) {
            eprintln!("{e}");
        }

        Ok(())
    }

    fn draw(&mut self, data: &mut SharedData, ctx: &mut Context) -> GameResult {
        let tile_size = self.world.map.tile_size;
        let aspect = self.world.aspect;

        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::WHITE);
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let mut array = ggez::graphics::InstanceArray::new(ctx, data.atlas.image.clone());

        for (id, pos) in self.world.map.static_tiles.iter() {
            array.push(
                DrawParam::default()
                    .src(registry().get_block_directly(*id).unwrap().uv.unwrap())
                    .dest(pos.as_vec2() * tile_size - self.player.camera.pos)
                    .scale(aspect),
            );
        }

        for (_, (trect, pos)) in data.ecs.query_mut::<(&crate::ecs::Textured, &Position)>() {
            array.push(
                DrawParam::default()
                    .src(trect.0)
                    .dest(
                        ggez::glam::vec2(pos.0 as f32, pos.1 as f32) * tile_size
                            - self.player.camera.pos,
                    )
                    .scale(aspect),
            );
        }

        array.draw(&mut canvas, DrawParam::default());
        canvas.draw(
            &ggez::graphics::Text::new(format!("FPS: {:.0}", ctx.time.fps())),
            DrawParam::default().color(ggez::graphics::Color::RED),
        );

        // should be last because of scissors
        self.player.draw(&mut canvas, &data.atlas)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn window_resize(
        &mut self,
        data: &mut SharedData,
        _ctx: &mut Context,
        new_width: f32,
        new_height: f32,
    ) -> GameResult {
        self.player
            .resize_event(&self.world.map, data, new_width, new_height);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        match input.keycode {
            Some(KeyCode::F11) => {
                data.settings.fullscreen_type = match data.settings.fullscreen_type {
                    FullscreenType::Windowed => FullscreenType::Desktop,
                    _ => FullscreenType::Windowed,
                };

                ctx.gfx.set_fullscreen(data.settings.fullscreen_type)?;
            }
            Some(KeyCode::Escape) => ctx.request_quit(),
            Some(key) => self.player.camera.key_down_event(key),
            None => {}
        }

        Ok(())
    }

    fn key_up_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
    ) -> GameResult {
        match input.keycode {
            Some(key) => self.player.camera.key_up_event(key),
            None => {}
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        button: MouseButton,
        mx: f32,
        my: f32,
    ) -> GameResult {
        match button {
            MouseButton::Left => {
                let index = self
                    .player
                    .ui
                    .block_list
                    .mouse_button_down_event(&data.settings, ggez::glam::Vec2::new(mx, my));

                if index != None {
                    self.cur_block = index;
                    return Ok(());
                }

                let (x, y) = self.point_to_block_pos(mx, my);
                if ctx.keyboard.is_key_pressed(KeyCode::RShift)
                    || ctx.keyboard.is_key_pressed(KeyCode::LShift)
                {
                    self.cur_net = self.get_block_network_id(&mut data.ecs, x, y);
                } else {
                    self.handle_click_on_block(data, x, y, ctx.time.delta().as_secs_f32())?;
                }
            }

            MouseButton::Middle => {
                let (x, y) = self.point_to_block_pos(mx, my);

                if let Some(entity) = self.world.map.get(x, y) {
                    if let Ok((id, network)) = data
                        .ecs
                        .query_one_mut::<(&BlockType, Option<&NetworkId>)>(entity)
                    {
                        self.cur_block = Some(id.0);
                        if let Some(net_id) = network {
                            self.cur_net = Some(net_id.0)
                        }
                    }
                }
            }

            MouseButton::Right => {
                let (x, y) = self.point_to_block_pos(mx, my);
                self.remove_block(&mut data.ecs, x, y);
            }

            MouseButton::Other(_) => {}
        }

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        button: MouseButton,
        mx: f32,
        my: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            let (x, y) = self.point_to_block_pos(mx, my);
            let index = self.world.map.index(x, y);

            if let Err(e) = data.script_engine.run_lua_function(
                &mut data.ecs,
                crate::LUA_FUNCTION_MOUSE_BUTTON_UP,
                &mut self.world,
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
        y: f32,
    ) -> GameResult {
        if !self
            .player
            .ui
            .block_list
            .scroll_event(&data.settings, ctx.mouse.position(), y)
        {
            self.world.aspect += y * 0.1;
            self.world.map.tile_size = crate::TEXTURE_SIZE * self.world.aspect.x;

            self.player.camera.resize_event(
                &self.world.map,
                data,
                data.settings.sc_width,
                data.settings.sc_height,
            );
        }

        Ok(())
    }
}
