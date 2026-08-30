use std::collections::HashMap;

use mlua::AnyUserData;

use crate::{
    defs::registry,
    ecs::{BlockType, Ecs, NetNode, Position},
    world::World,
};

mod param {
    // field names for scripts
    pub const STRING_ID: &str = "str_id";
    pub const BLOCK_INDEX_IN_REGISTRY: &str = "raw_id";
    pub const ENTITY_ID: &str = "entity_id";
    pub const NETWORK_ID: &str = "net_id";
    pub const POSITION: &str = "pos";
}

pub mod functions {
    pub const INIT: &str = "init";
    pub const UPDATE: &str = "update";
    pub const MOUSE_BUTTON_DOWN: &str = "on_mouse_button_down";
    pub const MOUSE_BUTTON_UP: &str = "on_mouse_button_up";
}

pub struct ScriptEngine {
    lua: mlua::Lua,
    scripts: HashMap<String, HashMap<u32, mlua::Function>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self {
            lua: mlua::Lua::new(),
            scripts: HashMap::new(),
        }
    }

    pub fn json_to_lua(&self, v: &serde_json::Value) -> mlua::Result<mlua::Value> {
        match v {
            serde_json::Value::Null => Ok(mlua::Value::Nil),
            serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return Ok(mlua::Value::Integer(i));
                } else {
                    return Ok(mlua::Value::Number(n.as_f64().ok_or(
                        mlua::Error::RuntimeError(format!("Invalid number {v}")),
                    )?));
                }
            }
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

    pub fn init_api(&mut self) -> mlua::Result<()> {
        let get_name = self.lua.create_function(move |_, raw_id: u32| {
            Ok(registry()
                .get_block_directly(raw_id)
                .or(Err(mlua::Error::RuntimeError(
                    "The block with this raw ID was not found".to_owned(),
                )))?
                .name
                .clone())
        })?;

        let get_block_str_id = self.lua.create_function(move |_, raw_id: u32| {
            Ok(registry()
                .get_block_directly(raw_id)
                .or(Err(mlua::Error::RuntimeError(
                    "The block with this raw ID was not found".to_owned(),
                )))?
                .id
                .clone())
        })?;

        let get_size = self.lua.create_function(move |_, raw_id: u32| {
            Ok(registry()
                .get_block_directly(raw_id)
                .or(Err(mlua::Error::RuntimeError(
                    "The block with this raw ID was not found".to_owned(),
                )))?
                .size
                .clone())
        })?;

        let get_entity_at =
            self.lua
                .create_function(move |_, (world, x, y): (AnyUserData, u16, u16)| {
                    world.borrow_scoped(|world: &World| {
                        if let Some(e) =
                            world.map.block_entities[(y * world.map.width + x) as usize]
                        {
                            return e.to_bits().get();
                        }

                        0
                    })
                })?;

        let get_entity_table =
            self.lua
                .create_function(move |lua, (ecs, entity): (AnyUserData, u64)| {
                    ecs.borrow_scoped(|ecs: &Ecs| {
                        if let Some(key) = &ecs
                            .get::<&crate::ecs::Table>(hecs::Entity::from_bits(entity).unwrap())
                            .expect("Entity not found")
                            .0
                        {
                            return Ok(mlua::Value::Table(
                                lua.registry_value::<mlua::Table>(&key)?,
                            ));
                        }

                        Ok(mlua::Value::Nil)
                    })?
                })?;

        let get_block_at =
            self.lua
                .create_function(move |lua, (world, x, y): (AnyUserData, u16, u16)| {
                    world.borrow_scoped(|world: &World| {
                        let table = lua.create_table()?;
                        let block = world.map.static_tiles[world.map.index(x, y)];

                        table.set(param::BLOCK_INDEX_IN_REGISTRY, block.0)?;
                        table.set(
                            param::STRING_ID,
                            registry()
                                .get_block_directly(block.0)
                                .or(Err(mlua::Error::RuntimeError(
                                    "The block with this raw ID was not found".to_owned(),
                                )))?
                                .id
                                .to_owned(),
                        )?;
                        table.set(param::POSITION, block.1.to_array())?;

                        Ok(table)
                    })?
                })?;

        let get_imbalance =
            self.lua
                .create_function(move |_, (world, net_id): (AnyUserData, u32)| {
                    world.borrow_scoped(|world: &World| {
                        let net = world.networks.get(&net_id);
                        match net {
                            Some(n) => mlua::Value::Number(n.get_storage_imbalance() as f64),
                            None => mlua::Value::Nil,
                        }
                    })
                })?;

        let get_world_width = self.lua.create_function(move |_, world: AnyUserData| {
            world.borrow_scoped(|world: &World| world.map.width)
        })?;

        let get_world_height = self.lua.create_function(move |_, world: AnyUserData| {
            world.borrow_scoped(|world: &World| world.map.height)
        })?;

        self.lua.globals().set("get_name", get_name)?;
        // Gets the block's string ID in the registry
        self.lua
            .globals()
            .set("get_block_str_id", get_block_str_id)?;
        self.lua.globals().set("get_size", get_size)?;
        self.lua.globals().set("get_entity_at", get_entity_at)?;
        self.lua
            .globals()
            .set("get_entity_table", get_entity_table)?;
        self.lua.globals().set("get_block_at", get_block_at)?;
        // Gets the total network imbalance divided by the number of energy storages
        self.lua.globals().set("get_imbalance", get_imbalance)?;
        self.lua.globals().set("get_world_width", get_world_width)?;
        self.lua
            .globals()
            .set("get_world_height", get_world_height)?;

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

        self.load_lua_function(&env, block_id, functions::INIT);
        self.load_lua_function(&env, block_id, functions::UPDATE);
        self.load_lua_function(&env, block_id, functions::MOUSE_BUTTON_DOWN);
        self.load_lua_function(&env, block_id, functions::MOUSE_BUTTON_UP);

        Ok(())
    }

    fn create_table(
        &self,
        entity: &hecs::Entity,
        id: &BlockType,
        pos: &Position,
        table: &mut crate::ecs::Table,
        node: Option<&NetNode>,
    ) -> mlua::Result<mlua::Table> {
        let block_table = self.lua.create_table()?;

        block_table.set(param::ENTITY_ID, entity.to_bits().get())?;
        block_table.set(param::BLOCK_INDEX_IN_REGISTRY, id.0)?;
        block_table.set(param::POSITION, vec![pos.0, pos.1])?;

        if let Some(n) = node {
            block_table.set(param::NETWORK_ID, n.0)?;
        }

        for (key, value) in &registry()
            .get_block_directly(id.0)
            .or(Err(mlua::Error::RuntimeError(
                "The block with this raw ID was not found".to_owned(),
            )))?
            .fields
        {
            block_table.set(key.to_owned(), self.json_to_lua(value)?)?;
        }

        table.0 = Some(self.lua.create_registry_value(block_table.clone())?);

        Ok(block_table)
    }

    pub fn run_lua_function(
        &mut self,
        ecs: &mut Ecs,
        name: &str,
        world: &mut crate::world::World,
        index: usize,
        dt: f32,
    ) -> mlua::Result<()> {
        let Some(func_groups) = self.scripts.get(name) else {
            return Ok(());
        };

        let Some(entity) = world.map.block_entities[index] else {
            return Ok(());
        };

        if let Ok((id, pos, table, node)) = ecs.query_one_mut::<(
            &BlockType,
            &Position,
            &mut crate::ecs::Table,
            Option<&NetNode>,
        )>(entity)
        {
            let Some(func) = func_groups.get(&id.0) else {
                return Ok(());
            };

            let table = if let Some(key) = &table.0 {
                let t = self.lua.registry_value::<mlua::Table>(&key)?;

                // Should to be updated every time, because the networks are rebuilt when a block is placed.
                if let Some(n) = node {
                    t.set(param::NETWORK_ID, n.0)?;
                }

                t
            } else {
                self.create_table(&entity, id, pos, table, node)?
            };

            self.lua.scope(|scope| {
                let world_ud = scope.create_any_userdata_ref(world)?;
                let ecs_ud = scope.create_any_userdata_ref(ecs)?;
                func.call::<()>((world_ud, ecs_ud, table, dt))
            })?;
        }

        Ok(())
    }

    pub fn run_update_function(
        &mut self,
        ecs: &mut Ecs,
        world: &mut World,
        dt: f32,
    ) -> mlua::Result<()> {
        let Some(func_group) = self.scripts.get(functions::UPDATE) else {
            return Ok(());
        };

        let mut block_groups: HashMap<u32, Vec<mlua::Table>> = HashMap::new();

        for (entity, (id, pos, table, node)) in ecs.query_mut::<(
            &BlockType,
            &Position,
            &mut crate::ecs::Table,
            Option<&NetNode>,
        )>() {
            if let Some(key) = &table.0 {
                block_groups
                    .entry(id.0)
                    .or_default()
                    .push(self.lua.registry_value(&key)?);
            } else {
                let table = self.create_table(&entity, id, pos, table, node)?;
                block_groups.entry(id.0).or_default().push(table);
            }
        }

        for (block_type, entities) in block_groups {
            if let Some(func) = func_group.get(&block_type) {
                self.lua.scope(|scope| {
                    let world_ud = scope.create_any_userdata_ref(world)?;
                    let ecs_ud = scope.create_any_userdata_ref(ecs)?;
                    func.call::<()>((world_ud, ecs_ud, entities, dt))
                })?;
            }
        }

        Ok(())
    }
}
