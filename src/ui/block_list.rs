use crate::{Settings, defs::registry, res::Atlas, res::TEXTURE_SIZE};
use ggez::{
    glam::Vec2,
    graphics::{Color, DrawParam, Rect},
};

static SCROLLBAR_COLOR: Color = Color::new(1.0, 1.0, 245.0 / 255.0, 1.0);

pub struct BlockListUI {
    hitbox: Rect,
    scrollbar: Rect,
    atlas_rect: Rect,
    aspect: Vec2,
    padding: f32,
    item_size: f32,
    scroll_offset: f32,
    bottom_border: f32,
    sizes: Vec<Vec2>,
    blocks: Vec<crate::defs::BlockDef>,
}

impl BlockListUI {
    const COLS: usize = 4;
    // Only natural values
    const PADDING: f32 = 8.0;
    const SCROLLBAR_WIDTH: f32 = 8.0;

    pub fn new(atlas: &Atlas, aspect: f32, settings: &Settings) -> Self {
        let blocks: Vec<crate::defs::BlockDef> = registry()
            .get_all_blocks()
            .iter()
            .filter(|def| !def.editor_only || settings.editor_mode)
            .map(|def| def.clone())
            .collect();

        let aspect = Vec2::splat(aspect * 2.0);
        let scrollbar_width = Self::SCROLLBAR_WIDTH * aspect.x;

        let hitbox_width = ((TEXTURE_SIZE + Self::PADDING) * Self::COLS as f32 + Self::PADDING)
            * aspect.x
            + scrollbar_width;
        let hitbox_height = hitbox_width - scrollbar_width;

        let hitbox_x = settings.screen_width - hitbox_width;
        let hitbox_y = settings.screen_height - hitbox_height;

        Self {
            hitbox: Rect::new(hitbox_x, hitbox_y, hitbox_width, hitbox_height),
            scrollbar: Rect::new(
                settings.screen_width - scrollbar_width,
                hitbox_y,
                scrollbar_width,
                // 16 because of 4 rows and 4 columns
                hitbox_height / (blocks.len().max(16) / 16) as f32,
            ),
            atlas_rect: *atlas.get_ui_uv::<Self>().unwrap(),
            aspect,
            padding: Self::PADDING * aspect.x,
            item_size: (TEXTURE_SIZE + Self::PADDING as f32) * aspect.x,
            scroll_offset: 0.0,
            bottom_border: ((blocks.len() / 16) as f32) * hitbox_height,
            sizes: blocks.iter().map(|def| aspect / def.size as f32).collect(),
            blocks,
        }
    }

    fn update_scrollbar_height(&mut self) {
        // 16 because of 4 rows and 4 columns
        self.scrollbar.h = self.hitbox.h / (self.blocks.len().max(16) / 16) as f32;
    }

    pub fn update_block_list(&mut self, settings: &Settings) {
        self.blocks = registry()
            .get_all_blocks()
            .iter()
            .filter(|def| !def.editor_only || settings.editor_mode)
            .map(|def| def.clone())
            .collect();
        self.sizes = self
            .blocks
            .iter()
            .map(|def| self.aspect / def.size as f32)
            .collect();
        self.update_scrollbar_height();
    }

    pub fn draw(
        &self,
        ctx: &ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        atlas: &Atlas,
    ) -> ggez::GameResult {
        let xy = self.hitbox.point();

        // draw background
        canvas.draw(
            &atlas.image,
            DrawParam::default()
                .src(self.atlas_rect)
                .dest(xy)
                .scale(self.aspect),
        );

        canvas.set_scissor_rect(self.hitbox)?;

        // draw scrollbar
        canvas.draw(
            &ggez::graphics::Mesh::new_rectangle(
                ctx,
                ggez::graphics::DrawMode::fill(),
                self.scrollbar,
                SCROLLBAR_COLOR,
            )?,
            DrawParam::new().dest(Vec2::new(0.0, -self.scroll_offset)),
        );

        // draw blocks
        for (i, def) in self.blocks.iter().enumerate() {
            canvas.draw(
                &atlas.image,
                DrawParam::default()
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

    pub fn mouse_motion_event(&mut self, sensitivity: f32, x: f32, y: f32, dy: f32) {
        if self.scrollbar.contains([x, y]) {
            let new_offset = self.scroll_offset - (dy * self.aspect.x * sensitivity);
            self.scroll_offset = new_offset.clamp(-self.bottom_border, 0.0);
        }
    }

    pub fn mouse_button_down_event(&self, settings: &Settings, mouse_pos: Vec2) -> Option<u32> {
        if self.hitbox.contains(mouse_pos) {
            let x = (self.hitbox.w - (settings.screen_width - mouse_pos.x)) as usize;
            let y = (self.hitbox.h - (settings.screen_height - mouse_pos.y + self.scroll_offset))
                as usize;

            let index = x / self.item_size as usize + (y / self.item_size as usize) * Self::COLS;
            if index >= self.blocks.len() {
                return None;
            }

            return registry()
                .get_block_index(&self.blocks[index as usize].id)
                .ok();
        }

        None
    }

    pub fn scroll(
        &mut self,
        sensitivity: f32,
        mouse_pos: impl Into<ggez::mint::Point2<f32>>,
        dy: f32,
    ) -> bool {
        let contains = self.hitbox.contains(mouse_pos);
        if contains {
            let new_offset = self.scroll_offset
                + (dy * (TEXTURE_SIZE + Self::PADDING) * self.aspect.x * sensitivity);
            self.scroll_offset = new_offset.clamp(-self.bottom_border, 0.0);
        }

        contains
    }
}

impl super::UI for BlockListUI {
    fn get_texture_path() -> &'static str {
        "./resources/assets/textures/ui/blocks_list.png"
    }

    fn resize_event(&mut self, new_width: f32, new_height: f32) {
        self.hitbox.x = new_width - self.hitbox.w;
        self.hitbox.y = new_height - self.hitbox.h;

        self.scrollbar.x = new_width - self.scrollbar.w;
        self.scrollbar.y = self.hitbox.y;
    }
}
