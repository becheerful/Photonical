use ggez::{
    Context,
    GameResult,
    event::{EventHandler, MouseButton},
    input::keyboard::KeyInput,
};

use crate::{
    Settings,
    player::Player,
    res::Atlas,
    scripts::ScriptEngine,
};

pub trait State {
    fn update(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
    ) -> GameResult;

    fn draw(
        &mut self,
        data: &SharedData,
        ctx: &mut Context,
    ) -> GameResult;

    fn window_resize(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        new_width: f32,
        new_height: f32,
    ) -> GameResult;

    fn key_down_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        input: KeyInput,
        repeated: bool,
    ) -> GameResult;

    fn mouse_button_down_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        button: MouseButton,
        mx: f32,
        my: f32,
    ) -> GameResult;

    fn mouse_button_up_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        button: MouseButton,
        mx: f32,
        my: f32,
    ) -> GameResult;

    fn mouse_wheel_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        x: f32,
        y: f32,
    ) -> GameResult;
}

pub struct SharedData {
    pub atlas: Atlas,
    pub script_engine: ScriptEngine,
    pub settings: Settings,
}

pub struct GameHandler {
    pub state: Box<dyn State>,
    pub data: SharedData,
}

impl GameHandler {
    pub fn new(
        atlas: Atlas,
        world: crate::world::World,
        player: Player,
        script_engine: ScriptEngine,
        settings: Settings,
    ) -> Self {
        GameHandler {
            state: Box::new(crate::states::playing::PlayingState::new(world, player)),
            data: SharedData {
                atlas,
                script_engine,
                settings,
            },
        }
    }
}

impl EventHandler for GameHandler {
    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, repeated: bool) -> GameResult {
        self.state.key_down_event(&mut self.data, ctx, input, repeated)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: ggez::event::MouseButton, mx: f32, my: f32) -> GameResult {
        self.state.mouse_button_down_event(&mut self.data, ctx, button, mx, my)?;
        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, mx: f32, my: f32) -> GameResult {
        self.state.mouse_button_up_event(&mut self.data, ctx, button, mx, my)?;
        Ok(())
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> GameResult {
        self.state.mouse_wheel_event(&mut self.data, ctx, x, y)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.state.update(&mut self.data, ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.state.draw(&self.data, ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.data.settings.sc_width = width;
        self.data.settings.sc_height = height;
        self.state.window_resize(&mut self.data, ctx, width, height)?;
        Ok(())
    }
}
