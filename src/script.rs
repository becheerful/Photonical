use std::{collections::HashMap};

use mlua::Lua;

use crate::{defs::registry, world::{BlockType, Position, Scripted, World}};

pub struct ScriptEngine {
    lua: Lua,
    scripts: HashMap<u32, mlua::Function>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: mlua::Lua::new(), scripts: HashMap::new() }
    }

    pub fn init_api(&self) -> mlua::Result<()> {
        let get_name = self.lua.create_function(move |_, block_id: u32| {
            Ok(registry().get_block_directly(block_id).unwrap().name.clone())
        })?;

        let get_block_id = self.lua.create_function(move |_, block_id: u32| {
            Ok(registry().get_block_directly(block_id).unwrap().id.clone())
        })?;

        self.lua.globals().set("get_name", get_name)?;
        self.lua.globals().set("get_block_id", get_block_id)?;

        Ok(())
    }

    pub fn load_script(&mut self, block_id: u32, code: &str) -> mlua::Result<()> {
        self.lua.load(code).exec()?;
        let func: mlua::Function = self.lua.globals().get("update")?;
        self.scripts.insert(block_id, func);
        Ok(())
    }

    pub fn update(&mut self, world: &mut World, dt: f32) -> mlua::Result<()> {
        // `u32` is an index of a block's identifier in the registry
        // `Vec<u32>` contains an identifier of each entity of this block type
        let mut groups: HashMap<u32, Vec<mlua::Table>> = HashMap::new();

        for (entity, (id, pos, _)) in world.ecs.query::<(&BlockType, &Position, &Scripted)>().iter() {
            let table = self.lua.create_table()?;
            table.set("entity_id", entity.id())?;
            table.set("block_id", id.0)?;
            table.set("pos", pos.0.to_array())?;
            groups.entry(id.0).or_default().push(table);
        }

        for (block_id, entities) in groups {
            let func = self.scripts.get(&block_id).unwrap();
            let blocks = self.lua.create_table()?;

            for (i, table) in entities.iter().enumerate() {
                blocks.set(i + 1, table)?;
            }

            func.call::<()>((blocks, dt))?;
        }

        Ok(())
    }
}
