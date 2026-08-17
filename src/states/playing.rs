use ggez::{
    Context, GameError, GameResult,
    event::MouseButton,
    glam::{UVec2, Vec2},
    graphics::{DrawParam, Drawable, InstanceArray},
};

use crate::{
    defs::{BlockDef, registry},
    ecs::{
        BlockType, LightBeam, NetworkId, Position, PowerConsumer, PowerProducer, PowerStorage,
        Table, UV,
    },
    game::SharedData,
    player::Player,
    settings::{CONNECTION_RADIUS, res::TEXTURE_SIZE},
    world::World,
};

pub struct PlayingState {
    cur_block: Option<u32>,
    static_layer: InstanceArray,
    dynamic_layer: InstanceArray,
    player: Player,
    world: World,
}

impl PlayingState {
    pub fn new(ctx: &Context, data: &mut SharedData) -> GameResult<Self> {
        let world = World::new(128, 128, 32.0)?;
        Ok(Self {
            cur_block: None,
            dynamic_layer: InstanceArray::new(ctx, data.atlas.image.clone()),
            static_layer: PlayingState::make_static_layer(ctx, &data.atlas.image, &world)?,
            player: Player::new(&world, registry(), &data.atlas, &data.settings),
            world,
        })
    }

    fn make_static_layer(
        ctx: &Context,
        image: &ggez::graphics::Image,
        world: &World,
    ) -> GameResult<InstanceArray> {
        let mut static_layer = InstanceArray::new(ctx, image.clone());
        for (id, pos) in world.map.static_tiles.iter() {
            static_layer.push(
                DrawParam::default()
                    .src(registry().get_block_directly(*id)?.uv.unwrap())
                    .dest(pos.as_vec2() * TEXTURE_SIZE),
            );
        }

        Ok(static_layer)
    }

    fn add_to_dynamic_layer(&mut self, uv: &UV, pos: &Position) {
        self.dynamic_layer.push(
            DrawParam::default()
                .src(uv.0)
                .dest(pos.to_vec2() * TEXTURE_SIZE),
        );
    }

    fn remove_from_dynamic_layer(&mut self, data: &mut SharedData) {
        self.dynamic_layer.clear();
        for (_, (uv, pos)) in data.ecs.query_mut::<(&UV, &Position)>() {
            self.dynamic_layer.push(
                DrawParam::default()
                    .src(uv.0)
                    .dest(pos.to_vec2() * TEXTURE_SIZE),
            );
        }
    }

    fn point_to_block_pos(&self, px: f32, py: f32) -> (u16, u16) {
        (
            ((px + self.player.camera.pos.x) / TEXTURE_SIZE / self.world.aspect) as u16,
            ((py + self.player.camera.pos.y) / TEXTURE_SIZE / self.world.aspect) as u16,
        )
    }

    fn place_block(&mut self, uv: &UV, pos: &Position, size: u16, e: hecs::Entity) {
        self.world.place_block(pos.0, pos.1, size, e);
        self.add_to_dynamic_layer(uv, pos);
    }

    fn remove_block(&mut self, data: &mut SharedData, x: u16, y: u16) -> GameResult {
        self.world.remove_entity(&mut data.ecs, x, y)?;
        self.remove_from_dynamic_layer(data);
        Ok(())
    }

    fn get_energy_interaction_value<T: crate::ecs::EnergyComponent>(
        bd: &BlockDef,
    ) -> GameResult<f32> {
        let parameter_name = T::get_energy_param_name();
        Ok(bd
            .net
            .get(parameter_name)
            .ok_or(GameError::ConfigError(
                format!(
                    "Missing parameter `{parameter_name}` for network mask {}",
                    T::get_network_mask()
                )
                .to_owned(),
            ))?
            .as_f64()
            .ok_or(GameError::CustomError(format!(
                "Invalid value for `{parameter_name}`"
            )))? as f32)
    }

    fn connect_with_network(&mut self, data: &mut SharedData, x: u16, y: u16, size: u16) -> u32 {
        let mut net_id = self.world.energy_master.networks.len() as u32;
        let mut connections: Vec<LightBeam> = Vec::new();

        let half = size / 2;
        let hx = (x + half).min(self.world.map.width);
        let hy = (y + half).min(self.world.map.height);

        let end_x = x + size;
        let end_y = y + size;

        for fx in end_x..(end_x + CONNECTION_RADIUS).min(self.world.map.width) {
            if let Some(entity) = self.world.map.get(fx, hy) {
                if let Ok(nid) = data.ecs.get::<&NetworkId>(entity) {
                    net_id = nid.0;

                    connections.push(LightBeam([
                        Vec2::new(end_x as f32, hy as f32 + 0.5) * TEXTURE_SIZE,
                        Vec2::new(fx as f32, hy as f32 + 0.5) * TEXTURE_SIZE,
                    ]));

                    break;
                }
            }
        }

        let mut cx = x.saturating_sub(1);
        while cx > x.saturating_sub(CONNECTION_RADIUS) {
            if let Some(entity) = self.world.map.get(cx, hy) {
                if let Ok(nid) = data.ecs.get::<&NetworkId>(entity) {
                    net_id = nid.0;

                    connections.push(LightBeam([
                        Vec2::new(cx as f32 + 1.0, hy as f32 + 0.5) * TEXTURE_SIZE,
                        Vec2::new(x as f32, hy as f32 + 0.5) * TEXTURE_SIZE,
                    ]));

                    break;
                }
            }

            cx -= 1;
        }

        for fy in end_y..(end_y + CONNECTION_RADIUS).min(self.world.map.height) {
            if let Some(entity) = self.world.map.get(hx, fy) {
                if let Ok(nid) = data.ecs.get::<&NetworkId>(entity) {
                    net_id = nid.0;

                    connections.push(LightBeam([
                        Vec2::new(hx as f32 + 0.5, end_y as f32) * TEXTURE_SIZE,
                        Vec2::new(hx as f32 + 0.5, fy as f32) * TEXTURE_SIZE,
                    ]));

                    break;
                }
            }
        }

        let mut cy = y.saturating_sub(1);
        while cy > y.saturating_sub(CONNECTION_RADIUS) {
            if let Some(entity) = self.world.map.get(hx, cy) {
                if let Ok(nid) = data.ecs.get::<&NetworkId>(entity) {
                    net_id = nid.0;

                    connections.push(LightBeam([
                        Vec2::new(hx as f32 + 0.5, cy as f32 + 1.0) * TEXTURE_SIZE,
                        Vec2::new(hx as f32 + 0.5, y as f32) * TEXTURE_SIZE,
                    ]));

                    break;
                }
            }

            cy -= 1;
        }

        for connection in connections {
            data.ecs.spawn((connection,));
        }

        net_id
    }

    fn add_energy_block<T: crate::ecs::EnergyComponent + hecs::Component>(
        &mut self,
        data: &mut SharedData,
        x: u16,
        y: u16,
        bd: &BlockDef,
        component: T,
    ) -> GameResult<bool> {
        if !self.world.check_for_space(x, y, bd.size) {
            return Ok(false);
        }

        let Some(cur_block) = self.cur_block else {
            return Ok(false);
        };

        let uv = data
            .atlas
            .make_texture_rect(&registry().get_block_directly(cur_block)?.texture)?;
        let pos = Position(x, y);

        let net_id = self.connect_with_network(data, x, y, bd.size);
        component.add_to_energy_master(net_id, &mut self.world.energy_master);

        let e = data
            .ecs
            .spawn((uv, BlockType(cur_block), pos, component, NetworkId(net_id)));

        self.place_block(&uv, &pos, bd.size, e);

        Ok(true)
    }

    fn add_neutral_energy_block(
        &mut self,
        data: &mut SharedData,
        x: u16,
        y: u16,
        bd: &BlockDef,
    ) -> GameResult<bool> {
        if !self.world.check_for_space(x, y, bd.size) {
            return Ok(false);
        }

        let Some(cur_block) = self.cur_block else {
            return Ok(false);
        };

        let uv = data
            .atlas
            .make_texture_rect(&registry().get_block_directly(cur_block)?.texture)?;
        let pos = Position(x, y);

        let net_id = self.connect_with_network(data, x, y, bd.size);

        let e = data
            .ecs
            .spawn((uv, BlockType(cur_block), pos, NetworkId(net_id)));

        self.place_block(&uv, &pos, bd.size, e);

        Ok(true)
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

            let bd = registry().get_block_directly(cur_block)?;
            let has_network = !bd.net.is_empty();

            if has_network
                && let Some(net_mask) = bd.net.get(crate::settings::json::fields::ENERGY_MASK)
            {
                match net_mask.as_u64().unwrap_or(0) as u8 {
                    crate::settings::json::mask::PRODUCER => {
                        if !self.add_energy_block::<PowerProducer>(
                            data,
                            x,
                            y,
                            bd,
                            PowerProducer(PlayingState::get_energy_interaction_value::<
                                PowerProducer,
                            >(bd)?),
                        )? {
                            return Ok(());
                        }
                    }

                    crate::settings::json::mask::CONSUMER => {
                        if !self.add_energy_block::<PowerConsumer>(
                            data,
                            x,
                            y,
                            bd,
                            PowerConsumer(PlayingState::get_energy_interaction_value::<
                                PowerConsumer,
                            >(bd)?),
                        )? {
                            return Ok(());
                        }
                    }

                    crate::settings::json::mask::STORAGE => {
                        if !self.add_energy_block::<PowerStorage>(data, x, y, bd, PowerStorage)? {
                            return Ok(());
                        }
                    }

                    crate::settings::json::mask::NODE => {
                        if !self.add_neutral_energy_block(data, x, y, bd)? {
                            return Ok(());
                        }
                    }

                    _ => {
                        return Err(ggez::GameError::ConfigError("No such mask".to_owned()));
                    }
                }
            }

            if bd.script.is_some() {
                if self.world.map.block_entities[index].is_none() {
                    let uv = data
                        .atlas
                        .make_texture_rect(&registry().get_block_directly(cur_block)?.texture)?;
                    let pos = Position(x, y);

                    let e = data.ecs.spawn((
                        uv.clone(),
                        BlockType(cur_block),
                        pos.clone(),
                        Table(None),
                    ));

                    self.place_block(&uv, &pos, bd.size, e);
                    self.world.place_block(x, y, bd.size, e);
                } else {
                    let e = self.world.map.block_entities[index].unwrap();
                    if let Err(e) = data.ecs.insert_one(e, Table(None)) {
                        eprintln!("{e}");
                    }
                }

                if let Err(e) = data.script_engine.run_lua_function(
                    &mut data.ecs,
                    crate::settings::lua::functions::INIT,
                    &mut self.world,
                    index,
                    dt,
                ) {
                    eprintln!("{e}");
                }
            } else if !has_network {
                self.world.map.static_tiles[index] = (cur_block, UVec2::new(x as u32, y as u32));
            }

            self.cur_block = None;
        } else {
            if let Err(e) = data.script_engine.run_lua_function(
                &mut data.ecs,
                crate::settings::lua::functions::MOUSE_BUTTON_DOWN,
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
        let aspect = ggez::glam::Vec2::splat(self.world.aspect);

        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::WHITE);
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let draw_param = DrawParam::default()
            .dest(-self.player.camera.pos)
            .scale(aspect);

        self.static_layer.draw(&mut canvas, draw_param);
        self.dynamic_layer.draw(&mut canvas, draw_param);

        for (_, beam) in data.ecs.query_mut::<&LightBeam>() {
            let l =
                ggez::graphics::Mesh::new_line(ctx, &beam.0, 8.0, ggez::graphics::Color::WHITE)?;
            canvas.draw(&l, draw_param);
        }

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
        data.settings.screen_width = new_width;
        data.settings.screen_height = new_height;

        self.player
            .resize_event(&self.world.map, new_width, new_height);

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let Some(key) = input.keycode {
            self.player.camera.key_down_event(key);
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
                self.handle_click_on_block(data, x, y, ctx.time.delta().as_secs_f32())?;
            }

            MouseButton::Middle => {
                let (x, y) = self.point_to_block_pos(mx, my);

                let Some(entity) = self.world.map.get(x, y) else {
                    return Ok(());
                };

                if let Ok(id) = data.ecs.get::<&BlockType>(entity) {
                    self.cur_block = Some(id.0);
                }
            }

            MouseButton::Right => {
                let (x, y) = self.point_to_block_pos(mx, my);
                self.remove_block(data, x, y)?;
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
                crate::settings::lua::functions::MOUSE_BUTTON_UP,
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
            self.world.aspect = (self.world.aspect
                + (y * data.settings.aspect_mouse_wheel_sensitivity))
                .clamp(1.0, 2.0);

            self.player.camera.resize_event(
                &self.world.map,
                data.settings.screen_width,
                data.settings.screen_height,
            );
        }

        Ok(())
    }
}
