use std::collections::HashMap;

use ggez::glam::Vec2;
use rhai::{AST, Engine, Scope};

use crate::world::{Block, World};

pub struct ScriptEngine {
    engine: Engine,
    scripts: HashMap<String, AST>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        engine.register_type::<Vec2>();

        engine.register_fn("get_block", |world: &World, x: usize, y: usize| {
            world.get(x, y).cloned()
        });

        engine.register_fn("wr", |text: &str| {
            println!("{}", text)
        });

        engine.register_fn("wr", |num: f32| {
            println!("{}", num);
        });

        engine.register_fn("wr", |v: Vec2| {
            println!("({} {})", v.x, v.y);
        });

        Self { engine, scripts: HashMap::new() }
    }

    pub fn load_script(&mut self, block_id: &str, script_code: &str) -> Result<(), rhai::ParseError> {
        let ast = self.engine.compile(script_code)?;
        self.scripts.insert(block_id.to_string(), ast);
        Ok(())
    }

    pub fn update_block(&mut self, block: &mut Block, dt: f32) -> Result<(), Box<rhai::EvalAltResult>> {
        let Some(ast) = self.scripts.get(&block.def.id) else {
            return Ok(())
        };

        self.engine.call_fn::<()>(&mut Scope::new(), ast, "on_tick", (dt, block.pos))?;
        Ok(())
    }
}
