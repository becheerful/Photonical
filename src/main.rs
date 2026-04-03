use std::path::PathBuf;

use ggez::{Context, ContextBuilder, GameResult, event, graphics::{Canvas, Color, DrawParam, Drawable, InstanceArray}};

mod world;
mod res;

struct Game {
    pub world: world::World,
    pub atlas: res::Atlas,
}

impl Game {
    fn new(ctx: &mut Context, atlas_path: &str, world: world::World) -> GameResult<Game> {
        let mut atlas = res::Atlas::new(ctx, atlas_path, world.tile_size, 2, 2)?;
        atlas.load_rects();
        Ok(Game {
            world,
            atlas,
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.world.update()?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(255, 255, 255));
        let mut array = InstanceArray::new(ctx, self.atlas.image.clone());
        for tile in &self.world.map {
            let rect = self.atlas.rects.get(tile.id as usize).unwrap();
            array.push(DrawParam::default().src(*rect).dest(tile.pos));
        }
        array.draw(&mut canvas, DrawParam::default());
        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let world = world::World::new(100, 60, 16);
    let sc_w = (world.width * world.tile_size) as f32;
    let sc_h = (world.height * world.tile_size) as f32;

    let (mut ctx, event_loop) = ContextBuilder::new("advent", "becheerful")
        .window_setup(ggez::conf::WindowSetup::default().title("Advent"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(sc_w, sc_h))
        .build()
        .unwrap();
    ctx.fs.mount(&PathBuf::from("./resources"), true);
    
    let game = Game::new(&mut ctx, "/atlas.png", world)?;

    event::run(ctx, event_loop, game);
}
