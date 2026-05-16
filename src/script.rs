use std::collections::HashMap;

use mlua::Lua;

use crate::world::Block;

pub struct ScriptEngine {
    lua: Lua,
    scripts: HashMap<String, mlua::Function>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: mlua::Lua::new(), scripts: HashMap::new() }
    }

    pub fn load_script(&mut self, block_id: &str, code: &str) -> Result<(), mlua::Error> {
        self.lua.load(code).exec()?;
        let func: mlua::Function = self.lua.globals().get("update")?;
        self.scripts.insert(block_id.to_owned(), func);
        Ok(())
    }

    pub fn execute(&self, block: &mut Block, dt: f32) -> Result<(), String> {
        let script = self.scripts
            .get(&block.def.id)
            .ok_or_else(|| format!("The block '{}' has no script", block.def.id))?;
        script.call::<()>((dt,)).map_err(|e| e.to_string())?;
        Ok(())
    }
}
