use crate::{Settings, res::Atlas};
use block_list::BlockListUI;

pub mod block_list;

pub trait UI {
    fn get_texture_path() -> &'static str;
    fn resize_event(&mut self, new_width: f32, new_height: f32);
}

pub struct PlayerUI {
    pub block_list: BlockListUI,
}

impl PlayerUI {
    pub fn new(atlas: &Atlas, aspect: f32, settings: &Settings) -> Self {
        Self {
            block_list: BlockListUI::new(atlas, aspect, settings),
        }
    }

    pub fn collect_ui_paths() -> Vec<String> {
        vec![BlockListUI::get_texture_path().to_owned()]
    }

    pub fn draw(
        &self,
        ctx: &ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        atlas: &Atlas,
    ) -> ggez::GameResult {
        self.block_list.draw(ctx, canvas, atlas)?;
        Ok(())
    }

    pub fn resize_event(&mut self, new_width: f32, new_height: f32) {
        self.block_list.resize_event(new_width, new_height);
    }
}
