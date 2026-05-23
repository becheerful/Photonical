use std::{collections::HashMap};

use mlua::Lua;

use crate::{defs::registry, world::{BlockType, Scripted, World}};

pub struct ScriptEngine {
    lua: Lua,
    scripts: HashMap<u32, mlua::Function>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: mlua::Lua::new(), scripts: HashMap::new() }
    }

    pub fn init_api(&self) -> mlua::Result<()> {
        Ok(())
    }

    pub fn load_script(&mut self, block_id: u32, code: &str) -> mlua::Result<()> {
        self.lua.load(code).exec()?;
        let func: mlua::Function = self.lua.globals().get("update")?;
        self.scripts.insert(block_id, func);
        Ok(())
    }

    pub fn update(&mut self, world: &mut World, dt: f32) -> mlua::Result<()> {
        let mut groups: HashMap<u32, Vec<u32>> = HashMap::new();

        for (entity, (id, _)) in world.ecs.query::<(&BlockType, &Scripted)>().iter() {
            groups.entry(id.0).or_default().push(entity.id());
        }

        for (script_id, entities) in groups {
            let func = self.scripts.get(&script_id).unwrap();
            let table = self.lua.create_table()?;

            for (i, entity) in entities.iter().enumerate() {
                table.set(i + 1, entity.to_owned())?;
            }

            func.call::<()>((table, dt))?;
        }

        Ok(())
    }
}
