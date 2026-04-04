use std::path::PathBuf;

use ggez::{Context, ContextBuilder, GameResult, event::{self, EventHandler, MouseButton}, graphics::{Canvas, Color, DrawParam, Drawable, InstanceArray, Sampler, Text}};

use crate::res::Atlas;

mod world;
mod res;
mod entity;
mod player;

struct Game {
    pub world: world::World,
    pub atlas: res::Atlas,
}

impl Game {
    fn new(_ctx: &mut Context, atlas: res::Atlas, world: world::World) -> GameResult<Game> {
        Ok(Game {
            world,
            atlas,
        })
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ctx.mouse.button_pressed(MouseButton::Right) {
            let mouse_point = ctx.mouse.position();
            let tile_x = mouse_point.x as usize / self.world.tile_size;
            let tile_y = mouse_point.y as usize / self.world.tile_size;
            let tile = self.world.get_mut(tile_x, tile_y).unwrap();
            tile.id = world::BlockType::Stone;
        } else if ctx.mouse.button_pressed(MouseButton::Left) {
            let mouse_point = ctx.mouse.position();
            let tile_x = mouse_point.x as usize / self.world.tile_size;
            let tile_y = mouse_point.y as usize / self.world.tile_size;
            let tile = self.world.get_mut(tile_x, tile_y).unwrap();
            tile.id = world::BlockType::Air;
        }

        self.world.update()?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
        canvas.set_sampler(Sampler::nearest_clamp());

        let mut array = InstanceArray::new(ctx, self.atlas.image.clone());

        for tile in &self.world.map {
            let rect = self.atlas.rects.get(tile.id as usize).unwrap();
            array.push(DrawParam::default().src(*rect).dest(tile.pos).scale(self.atlas.aspect));
        }

        array.draw(&mut canvas, DrawParam::default());
        canvas.draw(&Text::new(format!("FPS: {:.0}", ctx.time.fps())), DrawParam::default().color(Color::BLACK));

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("advent", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Advent"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(640.0, 480.0))
        .build()
        .unwrap();
    ctx.fs.mount(&PathBuf::from("./resources"), true);

    let world = world::World::new(100, 60, 32);

    let mut atlas = Atlas::new(&ctx, "/atlas.png", 16, 2, 2)?;
    atlas.load_rects();
    atlas.calc_aspect(&world);

    let game = Game::new(&mut ctx, atlas, world)?;

    event::run(ctx, event_loop, game);
}
