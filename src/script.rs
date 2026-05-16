use std::collections::HashMap;

use mlua::Lua;

use crate::world::Block;

pub struct ScriptEngine {
    lua: Lua,
    scripts: HashMap<u32, mlua::Function>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: mlua::Lua::new(), scripts: HashMap::new() }
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
}
