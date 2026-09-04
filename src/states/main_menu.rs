use ggez::{Context, GameResult};

use crate::game::SharedData;

#[allow(unused)]
struct MainMenuState;

impl super::State for MainMenuState {
    fn update(&mut self, _data: &mut SharedData, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _data: &mut SharedData, _ctx: &mut Context) -> GameResult {
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

    fn key_up_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _input: ggez::input::keyboard::KeyInput,
    ) -> GameResult {
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _data: &mut SharedData,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
        _dx: f32,
        _dy: f32,
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
