use ggez::{Context, GameResult};

use crate::game::{SharedData, State};

struct MainMenuState {}
impl State for MainMenuState {
    fn update(&mut self, _data: &mut SharedData, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _data: &SharedData, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn window_resize(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _new_width: f32,
        _new_height: f32,
    ) -> GameResult {
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        _mx: f32,
        _my: f32,
    ) -> GameResult {
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        _mx: f32,
        _my: f32,
    ) -> GameResult {
        Ok(())
    }

    fn mouse_wheel_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
    ) -> GameResult {
        Ok(())
    }
}
