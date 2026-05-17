use std::{cell::RefCell, collections::HashMap, rc::Rc};

use mlua::{Lua, UserData};

use crate::{defs::registry, world::{Block, World}};

pub type WorldRef = Rc<RefCell<World>>;

pub struct BlockRef {
    pub index: usize,
    pub world: WorldRef,
}

impl UserData for BlockRef {
    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_pos", |lua, this, ()| {
            let world = this.world.borrow();
            let pos = world.map[this.index].pos;
            let table = lua.create_table()?;
            table.set("x", pos.x)?;
            table.set("y", pos.y)?;
            Ok(table)
        });

        methods.add_method("get_id", |_, this, ()| {
            let world = this.world.borrow();
            Ok(registry().get_block_directly(world.map[this.index].id).unwrap().id.to_owned())
        });

        methods.add_method("get_name", |_, this, ()| {
            let world = this.world.borrow();
            Ok(registry().get_block_directly(world.map[this.index].id).unwrap().name.to_owned())
        });
    }
}

pub struct ScriptEngine {
    lua: Lua,
    scripts: HashMap<u32, mlua::Function>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: mlua::Lua::new(), scripts: HashMap::new() }
    }

    pub fn init_api(&self, world_ref: WorldRef) -> mlua::Result<()> {
        let get_block_at = self.lua.create_function(move |lua, (x, y): (usize, usize)| {
            let world = world_ref.borrow();
            let block_ref = BlockRef { index: y * world.width + x, world: world_ref.clone() };
            let ud = lua.create_userdata(block_ref)?;
            Ok(Some(ud))
        })?;

        self.lua.globals().set("get_block_at", get_block_at)?;

        Ok(())
    }

    pub fn load_script(&mut self, block_id: u32, code: &str) -> Result<(), mlua::Error> {
        self.lua.load(code).exec()?;
        let func: mlua::Function = self.lua.globals().get("update")?;
        self.scripts.insert(block_id, func);
        Ok(())
    }

    pub fn execute(&self, block: &mut Block, dt: f32) -> Result<(), String> {
        // We can call `.unwrap()` here because we get a block from a world
        // If the world contains this block, therefore, it exists in the registry
        let script = self.scripts
            .get(&block.id)
            .ok_or_else(|| format!("The block '{}' has no script", block.id))?;
        script.call::<()>((dt,)).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update(&mut self, world_ref: WorldRef, dt: f32) -> mlua::Result<()> {
        let world = &world_ref.borrow();
        for mechanism in &world.mechanisms {
            let block = &world.map[*mechanism];
            let index = block.pos.y as usize * world.width + block.pos.x as usize;
            let block_ref = BlockRef { index, world: world_ref.clone() };
            let ud = self.lua.create_userdata(block_ref)?;
            let func = self.scripts.get(&block.id).unwrap();
            func.call::<()>((ud, dt))?;
        }

        Ok(())
    }
}
