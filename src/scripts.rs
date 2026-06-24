use std::collections::HashMap;

use crate::{
    LUA_FUNCTION_MOUSE_BUTTON_DOWN,
    LUA_FUNCTION_MOUSE_BUTTON_UP,
    LUA_FUNCTION_UPDATE,
    PARAM_BLOCK_INDEX_IN_REGISTRY,
    PARAM_ENTITY_ID,
    PARAM_NETWORK_ID,
    PARAM_POSITION,
    defs::registry,
    world::{BlockType, NetworkId, Position}
};

pub struct ScriptEngine {
    lua: mlua::Lua,
    scripts: HashMap<String, HashMap<u32, mlua::Function>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self {
            lua: mlua::Lua::new(),
            scripts: HashMap::new()
        }
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
        let get_entity_at = self.lua.create_function(move |lua, (x, y): (u16, u16)| {
            let world = world.borrow();
            let entity = world.map.get(x, y);
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
            let block = world.map.static_tiles[world.map.index(x, y)];
            table.set(PARAM_BLOCK_INDEX_IN_REGISTRY, block.0)?;
            table.set(PARAM_POSITION, block.1.to_array())?;
            Ok(table)
        })?;

        let world = world_ref.clone();
        let get_imbalance = self.lua.create_function(move |_, net_id: u32| {
            let world = world.borrow();
            let net = world.energy_master.networks.get(&net_id);

            match net {
                Some(n) => Ok(mlua::Value::Integer(n.get_storage_imbalance())),
                None => Ok(mlua::Value::Nil),
            }
        })?;

        self.lua.globals().set("get_name", get_name)?;
        // Gets the block's string ID in the registry
        self.lua.globals().set("get_block_id", get_block_id)?;
        self.lua.globals().set("get_entity_at", get_entity_at)?;
        self.lua.globals().set("get_block_at", get_block_at)?;
        // Gets the total network imbalance divided by the number of energy storages
        self.lua.globals().set("get_imbalance", get_imbalance)?;

        Ok(())
    }

    fn load_lua_function(&mut self, env: &mlua::Table, block_id: u32, name: &str) {
        if let Ok(v) = env.get::<mlua::Value>(name) {
            if let Some(f) = v.as_function() {
                self.scripts
                    .entry(name.to_owned())
                    .or_insert(HashMap::new())
                    .insert(block_id, f.to_owned());
            }
        }
    }

    pub fn load_scripts(&mut self, block_id: u32, code: &str) -> mlua::Result<()> {
        let mtable = self.lua.create_table()?;
        mtable.set("__index", &self.lua.globals())?;

        let env = self.lua.create_table()?;
        env.set_metatable(Some(mtable));

        self.lua.load(code).set_environment(env.clone()).exec()?;

        self.load_lua_function(&env, block_id, LUA_FUNCTION_UPDATE);
        self.load_lua_function(&env, block_id, LUA_FUNCTION_MOUSE_BUTTON_DOWN);
        self.load_lua_function(&env, block_id, LUA_FUNCTION_MOUSE_BUTTON_UP);

        Ok(())
    }

    fn create_table(&self,
        entity: &hecs::Entity,
        id: &BlockType,
        pos: &Position,
        table: &mut crate::world::Table,
        network: Option<&NetworkId>,
    ) -> mlua::Result<mlua::Table> {
        let block_table = self.lua.create_table()?;

        block_table.set(PARAM_ENTITY_ID, entity.id())?;
        block_table.set(PARAM_BLOCK_INDEX_IN_REGISTRY, id.0)?;
        block_table.set(PARAM_POSITION, pos.0.to_array())?;

        if let Some(net_id) = network {
            block_table.set(PARAM_NETWORK_ID, net_id.0)?;
        }

        for (key, value) in &registry().get_block_directly(id.0).unwrap().fields {
            block_table.set(key.to_owned(), self.json_to_lua(value)?)?;
        }

        table.0 = Some(self.lua.create_registry_value(block_table.clone())?);

        Ok(block_table)
    }

    pub fn run_lua_function(&mut self, name: &str, world: &mut crate::world::World, index: usize, dt: f32) -> mlua::Result<()> {
        if let Some(func_groups) = self.scripts.get(name) {
            let Some(entity) = world.map.block_entities[index] else {
                return Ok(());
            };

            if let Ok((id, pos, table, network)) = world.ecs.query_one_mut::<(
                &BlockType, &Position, &mut crate::world::Table, Option<&NetworkId>,
            )>(entity) {
                if let Some(func) = func_groups.get(&id.0) {
                    let table = if let Some(key) = &table.0 {
                        self.lua.registry_value::<mlua::Table>(&key)?
                    } else {
                        self.create_table(&entity, id, pos, table, network)?
                    };

                    func.call::<()>((table, dt))?;
                }
            }
        }

        Ok(())
    }

    pub fn update(&mut self, world: &mut crate::world::World, dt: f32) -> mlua::Result<()> {
        if let Some(func_group) = self.scripts.get(LUA_FUNCTION_UPDATE) {
            let mut block_groups: HashMap<u32, Vec<mlua::Table>> = HashMap::new();

            for (entity, (id, pos, table, network)) in world.ecs.query_mut::<(
                &BlockType, &Position, &mut crate::world::Table, Option<&NetworkId>
            )>() {
                if let Some(key) = &table.0 {
                    block_groups.entry(id.0).or_default().push(self.lua.registry_value(&key)?);
                } else {
                    let table = self.create_table(&entity, id, pos, table, network)?;
                    block_groups.entry(id.0).or_default().push(table);
                }
            }

            for (block_type, entities) in block_groups {
                if let Some(func) = func_group.get(&block_type) {
                    func.call::<()>((entities, dt))?;
                }
            }
        }

        Ok(())
    }
}
