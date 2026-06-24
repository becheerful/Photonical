use ggez::{GameResult, glam::Vec2, graphics::Rect};

use crate::{Settings, res::Atlas};

pub trait UI {
    fn get_texture_path(&self) -> &str;
    fn resize_event(&mut self, new_width: f32, new_height: f32);
}

pub struct PlayerUI {
    pub block_list: BlockListUI,
}

impl PlayerUI {
    pub fn new(registry: &crate::defs::Registry, world: &crate::world::World, settings: &Settings) -> Self {
        Self {
            block_list: BlockListUI::new(registry, world, settings)
        }
    }

    pub fn collect_ui_paths(&self) -> Vec<String> {
        vec![
            self.block_list.get_texture_path().to_owned(),
        ]
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
    pub hitbox: Rect,
    pub atlas_rect: Option<Rect>,
    pub aspect: Vec2,
    pub padding: f32,
    pub item_size: f32,
    pub scroll_offset: f32,
    pub blocks: Vec<crate::defs::BlockDef>,
}

impl BlockListUI {
    const COLS: usize = 4;
    // Basically intended to be used as `u32`
    pub const PADDING: f32 = 8.0;

    pub fn new(registry: &crate::defs::Registry, world: &crate::world::World, settings: &Settings) -> Self {
        let aspect = world.aspect * 2.0;
        let width = ((crate::TEXTURE_SIZE + Self::PADDING) * Self::COLS as f32 + Self::PADDING) * aspect.x;

        Self {
            hitbox: Rect::new(settings.sc_width - width, settings.sc_height - width, width, width),
            atlas_rect: None,
            aspect,
            padding: Self::PADDING * world.aspect.x,
            item_size: (crate::TEXTURE_SIZE + Self::PADDING as f32) * 2.0,
            scroll_offset: 0.0,
            blocks: registry.get_all_blocks(),
        }
    }

    pub fn load_atlas_rect(&mut self, atlas: &Atlas) -> GameResult {
        self.atlas_rect = Some(*atlas.get_ui_uv(self)?);
        Ok(())
    }

    pub fn draw(&self, canvas: &mut ggez::graphics::Canvas, atlas: &Atlas) -> GameResult {
        let xy = self.hitbox.point();

        canvas.draw(
            &atlas.image,
            ggez::graphics::DrawParam::default()
                .src(self.atlas_rect.unwrap())
                .dest(xy)
                .scale(self.aspect)
        );

        for i in 0..self.blocks.len() {
            canvas.draw(
                &atlas.image,
                ggez::graphics::DrawParam::default()
                    .src(*atlas.get_block_uv(&self.blocks[i])?)
                    .dest(Vec2::new(
                        xy.x + (i % Self::COLS) as f32 * self.item_size + self.padding,
                        xy.y + (i / Self::COLS) as f32 * self.item_size + self.padding + self.scroll_offset,
                    ))
                    .scale(self.aspect)
            );
        }

        Ok(())
    }

    pub fn mouse_button_down_event(&self, mouse_pos: Vec2) -> bool {
        let contains = self.hitbox.contains(mouse_pos);
        if contains {
            // todo: choose block
        }

        contains
    }

    pub fn scroll_event(&mut self, settings: &Settings, mouse_pos: ggez::mint::Point2<f32>, dy: f32) -> bool {
        let contains = self.hitbox.contains(mouse_pos);
        if contains {
            self.scroll_offset += dy * (crate::TEXTURE_SIZE + Self::PADDING) * settings.mouse_wheel_sensitivity;
        }

        contains
    }
}

impl UI for BlockListUI {
    fn get_texture_path(&self) -> &str {
        "./resources/assets/textures/ui/blocks_list.png"
    }

    fn resize_event(&mut self, new_width: f32, new_height: f32) {
        let width = self.hitbox.w;
        self.hitbox.x = new_width - width;
        self.hitbox.y = new_height - width;
    }
}
