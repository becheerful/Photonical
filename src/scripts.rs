use std::collections::HashMap;

use mlua::Lua;

use crate::{
    PARAM_BLOCK_INDEX_IN_REGISTRY,
    PARAM_ENTITY_ID,
    PARAM_POSITION,
    defs::registry
};

pub struct ScriptEngine {
    lua: Lua,
    scripts: HashMap<u32, mlua::Function>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: mlua::Lua::new(), scripts: HashMap::new() }
    }

    pub fn json_to_lua(&self, v: &serde_json::Value) -> mlua::Result<mlua::Value> {
        match v {
            serde_json::Value::Null => Ok(mlua::Value::Nil),
            serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
            serde_json::Value::Number(n) => {
                if let Some(u) = n.as_i64() {
                    return Ok(mlua::Value::Integer(u));
                } else {
                    return Ok(mlua::Value::Number(n.as_f64().unwrap_or(0.0)));
                }
            },
            serde_json::Value::String(s) => Ok(mlua::Value::String(self.lua.create_string(s)?)),
            serde_json::Value::Array(v) => {
                let table = self.lua.create_table()?;
                for (i, item) in v.iter().enumerate() {
                    table.set(i + 1, self.json_to_lua(item)?)?;
                }

                Ok(mlua::Value::Table(table))
            }
            serde_json::Value::Object(m) => {
                let table = self.lua.create_table()?;
                for (k, v) in m {
                    table.set(k.to_owned(), self.json_to_lua(v)?)?;
                }

                Ok(mlua::Value::Table(table))
            }
        }
    }

    pub fn init_api(&mut self, world_ref: crate::WorldRef) -> mlua::Result<()> {
        let get_name = self.lua.create_function(move |_, index: u32| {
            Ok(registry().get_block_directly(index).unwrap().name.clone())
        })?;

        let get_block_id = self.lua.create_function(move |_, index: u32| {
            Ok(registry().get_block_directly(index).unwrap().id.clone())
        })?;

        let world = world_ref.clone();
        let get_mechanism_at = self.lua.create_function(move |lua, (x, y): (u16, u16)| {
            let world = world.borrow();
            let entity = world.get(x, y);
            if let Some(e) = entity {
                if let Some(key) = &world.ecs.get::<&crate::world::Table>(e).expect("Entity not found").0 {
                    return Ok(lua.registry_value(key)?);
                }
            }
            return Ok(mlua::Value::Nil);
        })?;

        let world = world_ref.clone();
        let get_block_at = self.lua.create_function(move |lua, (x, y): (u16, u16)| {
            let world = world.borrow();
            let table = lua.create_table()?;
            let block = world.static_tiles[world.index(x, y)];
            table.set(PARAM_BLOCK_INDEX_IN_REGISTRY, block.0)?;
            table.set(PARAM_POSITION, block.1.to_array())?;
            Ok(table)
        })?;

        self.lua.globals().set("get_name", get_name)?;
        self.lua.globals().set("get_block_id", get_block_id)?;
        self.lua.globals().set("get_mechanism_at", get_mechanism_at)?;
        self.lua.globals().set("get_block_at", get_block_at)?;

        Ok(())
    }

    pub fn load_script(&mut self, block_id: u32, code: &str) -> mlua::Result<()> {
        self.lua.load(code).exec()?;
        let func: mlua::Function = self.lua.globals().get("update")?;
        self.scripts.insert(block_id, func);
        Ok(())
    }

    pub fn update(&mut self, world: &crate::world::World, dt: f32) -> mlua::Result<()> {
        // `u32` is an index of a block's identifier in the registry
        // `Vec<u32>` contains an identifier of each entity of this block type
        let mut groups: HashMap<u32, Vec<mlua::Table>> = HashMap::new();

        for (entity, (id, pos, table)) in world.ecs.query::<(
            &crate::world::BlockType, &crate::world::Position, &mut crate::world::Table,
        )>().iter() {
            if let Some(key) = &table.0 {
                groups.entry(id.0).or_default().push(self.lua.registry_value(key)?);
            } else {
                let block_table = self.lua.create_table()?;
                block_table.set(PARAM_ENTITY_ID, entity.id())?;
                block_table.set(PARAM_BLOCK_INDEX_IN_REGISTRY, id.0)?;
                block_table.set(PARAM_POSITION, pos.0.to_array())?;

                for (key, value) in &registry().get_block_directly(id.0).unwrap().fields {
                    block_table.set(key.to_owned(), self.json_to_lua(value)?)?;
                }

                groups.entry(id.0).or_default().push(block_table.clone());
                table.0 = Some(self.lua.create_registry_value(block_table)?);
            }
        }

        for (block_id, entities) in groups {
            let func = self.scripts.get(&block_id).unwrap();
            func.call::<()>((entities, dt))?;
        }

        Ok(())
    }
}
