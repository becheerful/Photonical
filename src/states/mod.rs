use ggez::{Context, GameResult, event::MouseButton, input::keyboard::KeyInput};

use crate::game::SharedData;

pub mod main_menu;
pub mod playing;

pub trait State {
    fn update(&mut self, data: &mut SharedData, ctx: &mut Context) -> GameResult;

    fn draw(&mut self, data: &mut SharedData, ctx: &mut Context) -> GameResult;

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

    fn key_up_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        input: KeyInput,
    ) -> GameResult;

    fn mouse_motion_event(
        &mut self,
        data: &mut SharedData,
        ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
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
