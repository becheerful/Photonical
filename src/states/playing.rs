use ggez::{
    Context, GameResult,
    event::MouseButton,
    glam::UVec2,
    graphics::{Color, DrawParam, Drawable, InstanceArray},
};

use crate::{
    defs::{BlockDef, registry},
    ecs::{
        BlockType, LightBeam, LightProperties, NetNode, Position, PowerConsumer, PowerProducer,
        PowerStorage, Table, UV,
    },
    game::SharedData,
    json::get_wavelength,
    network::LightColor,
    player::Player,
    res::TEXTURE_SIZE,
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
        let world = World::new(128, 128)?;
        Ok(Self {
            cur_block: None,
            dynamic_layer: InstanceArray::new(ctx, data.atlas.image.clone()),
            static_layer: PlayingState::make_static_layer(ctx, &data.atlas.image, &world)?,
            player: Player::new(&world, &data.atlas, &data.settings),
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

    fn place_block(
        &mut self,
        ecs: &mut crate::ecs::Ecs,
        uv: &UV,
        pos: &Position,
        size: u16,
        e: hecs::Entity,
    ) {
        self.world.place_block(pos.0, pos.1, size, e);
        crate::network::rebuild_networks(&mut self.world, ecs);
        self.add_to_dynamic_layer(uv, pos);
    }

    fn remove_block(&mut self, data: &mut SharedData, x: u16, y: u16) -> GameResult {
        self.world.remove_entity(&mut data.ecs, x, y)?;
        crate::network::rebuild_networks(&mut self.world, &mut data.ecs);
        self.remove_from_dynamic_layer(data);
        Ok(())
    }

    fn add_energy_block<T: crate::ecs::EnergyComponent + hecs::Component>(
        &mut self,
        data: &mut SharedData,
        x: u16,
        y: u16,
        bd: &BlockDef,
        light_color: LightColor,
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

        let e = data.ecs.spawn((
            uv,
            BlockType(cur_block),
            pos,
            component,
            LightProperties(light_color),
            NetNode(crate::network::PLUG),
        ));

        self.place_block(&mut data.ecs, &uv, &pos, bd.size, e);
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

        let e = data.ecs.spawn((
            uv,
            BlockType(cur_block),
            pos,
            LightProperties(LightColor::Undefined),
            NetNode(crate::network::PLUG),
        ));

        self.place_block(&mut data.ecs, &uv, &pos, bd.size, e);
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

        if self.world.map.block_entities[index].is_some() {
            if let Err(e) = data.script_engine.run_lua_function(
                &mut data.ecs,
                crate::scripts::functions::MOUSE_BUTTON_DOWN,
                &mut self.world,
                index,
                dt,
            ) {
                eprintln!("{e}");
            }

            return Ok(());
        }

        let Some(cur_block) = self.cur_block else {
            return Ok(());
        };

        let bd = registry().get_block_directly(cur_block)?;
        let has_network = !bd.net.is_empty();

        if has_network && let Some(net_mask) = bd.net.get(crate::json::fields::ENERGY_MASK) {
            match net_mask.as_u64().unwrap_or(0) as u8 {
                crate::network::mask::PRODUCER => {
                    let power = crate::json::get_energy_interaction_value::<PowerProducer>(bd)?;
                    if !self.add_energy_block::<PowerProducer>(
                        data,
                        x,
                        y,
                        bd,
                        LightColor::from(get_wavelength(bd)?),
                        PowerProducer(power),
                    )? {
                        return Ok(());
                    }
                }

                crate::network::mask::CONSUMER => {
                    let demand = crate::json::get_energy_interaction_value::<PowerConsumer>(bd)?;
                    if !self.add_energy_block::<PowerConsumer>(
                        data,
                        x,
                        y,
                        bd,
                        LightColor::from(get_wavelength(bd)?),
                        PowerConsumer(demand),
                    )? {
                        return Ok(());
                    }
                }

                crate::network::mask::STORAGE => {
                    if !self.add_energy_block::<PowerStorage>(
                        data,
                        x,
                        y,
                        bd,
                        LightColor::Undefined,
                        PowerStorage,
                    )? {
                        return Ok(());
                    }
                }

                crate::network::mask::NODE => {
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

                let e =
                    data.ecs
                        .spawn((uv.clone(), BlockType(cur_block), pos.clone(), Table(None)));

                self.place_block(&mut data.ecs, &uv, &pos, bd.size, e);
            } else {
                let e = self.world.map.block_entities[index].unwrap();
                if let Err(e) = data.ecs.insert_one(e, Table(None)) {
                    eprintln!("{e}");
                }
            }

            if let Err(e) = data.script_engine.run_lua_function(
                &mut data.ecs,
                crate::scripts::functions::INIT,
                &mut self.world,
                index,
                dt,
            ) {
                eprintln!("{e}");
            }
        } else if !has_network {
            self.world.map.static_tiles[index] = (cur_block, UVec2::new(x as u32, y as u32));
        }

        Ok(())
    }
}

impl crate::states::State for PlayingState {
    fn update(&mut self, data: &mut SharedData, ctx: &mut Context) -> GameResult {
        self.player.camera.update();

        if let Err(e) = data.script_engine.run_update_function(
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

        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, Color::WHITE);
        canvas.set_sampler(ggez::graphics::Sampler::nearest_clamp());

        let draw_param = DrawParam::default()
            .dest(-self.player.camera.pos)
            .scale(aspect);

        self.static_layer.draw(&mut canvas, draw_param);
        self.dynamic_layer.draw(&mut canvas, draw_param);

        for (_, beam) in data.ecs.query_mut::<&LightBeam>() {
            let l = ggez::graphics::Mesh::new_line(ctx, &beam.0, 8.0, Color::WHITE)?;
            canvas.draw(&l, draw_param);
        }

        if data.settings.show_fps {
            canvas.draw(
                ggez::graphics::Text::new(format!("FPS: {:.0}", ctx.time.fps()))
                    .set_font("ScienceGothic")
                    .set_scale(36.0),
                DrawParam::default().color(Color::BLACK),
            );
        }

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
                if self.cur_block.is_some() {
                    self.cur_block = None;
                } else {
                    let (x, y) = self.point_to_block_pos(mx, my);
                    self.remove_block(data, x, y)?;
                }
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
                crate::scripts::functions::MOUSE_BUTTON_UP,
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
