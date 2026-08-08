use ggez::{GameResult, glam::Vec2, graphics::Rect};

use crate::{Settings, res::Atlas};

pub trait UI {
    fn get_texture_path() -> &'static str;
    fn resize_event(&mut self, new_width: f32, new_height: f32);
}

pub struct PlayerUI {
    pub block_list: BlockListUI,
}

impl PlayerUI {
    pub fn new(
        registry: &crate::defs::Registry,
        atlas: &Atlas,
        aspect: &Vec2,
        settings: &Settings,
    ) -> Self {
        Self {
            block_list: BlockListUI::new(registry, atlas, aspect, settings),
        }
    }

    pub fn collect_ui_paths() -> Vec<String> {
        vec![BlockListUI::get_texture_path().to_owned()]
    }

    pub fn draw(&self, canvas: &mut ggez::graphics::Canvas, atlas: &Atlas) -> GameResult {
        self.block_list.draw(canvas, atlas)?;
        Ok(())
    }

    pub fn resize_event(&mut self, new_width: f32, new_height: f32) {
        self.block_list.resize_event(new_width, new_height);
    }
}

pub struct BlockListUI {
    hitbox: Rect,
    atlas_rect: Rect,
    aspect: Vec2,
    padding: f32,
    item_size: f32,
    scroll_offset: f32,
    sizes: Vec<Vec2>,
    blocks: Vec<crate::defs::BlockDef>,
}

impl BlockListUI {
    const COLS: usize = 4;
    // Basically intended to be used as `u32`
    pub const PADDING: f32 = 8.0;

    pub fn new(
        registry: &crate::defs::Registry,
        atlas: &Atlas,
        aspect: &Vec2,
        settings: &Settings,
    ) -> Self {
        let aspect = *aspect * 2.0;
        let width =
            ((crate::TEXTURE_SIZE + Self::PADDING) * Self::COLS as f32 + Self::PADDING) * aspect.x;

        let blocks: Vec<crate::defs::BlockDef> = registry
            .get_all_blocks()
            .iter()
            .filter(|def| !def.editor_only)
            .map(|def| def.clone())
            .collect();

        Self {
            hitbox: Rect::new(
                settings.sc_width - width,
                settings.sc_height - width,
                width,
                width,
            ),
            atlas_rect: *atlas.get_ui_uv::<Self>().unwrap(),
            aspect,
            padding: Self::PADDING * aspect.x,
            item_size: (crate::TEXTURE_SIZE + Self::PADDING as f32) * aspect.x,
            scroll_offset: 0.0,
            sizes: blocks.iter().map(|def| aspect / def.size as f32).collect(),
            blocks,
        }
    }

    pub fn draw(&self, canvas: &mut ggez::graphics::Canvas, atlas: &Atlas) -> GameResult {
        let xy = self.hitbox.point();

        canvas.draw(
            &atlas.image,
            ggez::graphics::DrawParam::default()
                .src(self.atlas_rect)
                .dest(xy)
                .scale(self.aspect),
        );

        canvas.set_scissor_rect(self.hitbox)?;

        for (i, def) in self.blocks.iter().enumerate() {
            canvas.draw(
                &atlas.image,
                ggez::graphics::DrawParam::default()
                    .src(def.uv.unwrap())
                    .dest(Vec2::new(
                        xy.x + (i % Self::COLS) as f32 * self.item_size + self.padding,
                        xy.y + (i / Self::COLS) as f32 * self.item_size
                            + self.padding
                            + self.scroll_offset,
                    ))
                    .scale(self.sizes[i]),
            );
        }

        Ok(())
    }

    pub fn mouse_button_down_event(&self, settings: &Settings, mouse_pos: Vec2) -> Option<u32> {
        if self.hitbox.contains(mouse_pos) {
            let x = (self.hitbox.w - (settings.sc_width - mouse_pos.x)) as usize;
            let y =
                (self.hitbox.h - (settings.sc_height - mouse_pos.y + self.scroll_offset)) as usize;

            let index = x / self.item_size as usize + (y / self.item_size as usize) * Self::COLS;
            if index >= self.blocks.len() {
                return None;
            }

            return crate::defs::registry().get_block_index(&self.blocks[index].id);
        }

        None
    }

    pub fn scroll_event(
        &mut self,
        settings: &Settings,
        mouse_pos: ggez::mint::Point2<f32>,
        dy: f32,
    ) -> bool {
        let contains = self.hitbox.contains(mouse_pos);
        if contains {
            self.scroll_offset +=
                dy * (crate::TEXTURE_SIZE + Self::PADDING) * settings.mouse_wheel_sensitivity;
        }

        contains
    }
}

impl UI for BlockListUI {
    fn get_texture_path() -> &'static str {
        "./resources/assets/textures/ui/blocks_list.png"
    }

    fn resize_event(&mut self, new_width: f32, new_height: f32) {
        let width = self.hitbox.w;
        self.hitbox.x = new_width - width;
        self.hitbox.y = new_height - width;
    }
}
