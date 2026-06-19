use std::{collections::HashMap, fs};

use ggez::GameResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: String,
    pub name: String,
    pub texture: String,
    pub script: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
    pub net: serde_json::Map<String, serde_json::Value>,
    /// position in the dynamically stitched texture atlas \
    /// it's safe to use `.unwrap()` after the atlas is initialized
    #[serde(skip)]
    pub uv: Option<ggez::graphics::Rect>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub texture: String,
    /// position in the dynamically stitched texture atlas \
    /// it's safe to use `.unwrap()` after the atlas is initialized
    #[serde(skip)]
    pub uv: Option<ggez::graphics::Rect>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDef {
    pub id: String,
    pub time: f32,
    pub inputs: Vec<(String, u32)>,
    pub outputs: Vec<(String, u32)>,
}

#[derive(Debug)]
pub struct Registry {
    /// Matches the block ID with its position in `Vec<BlockDef>`
    blocks_idx: HashMap<String, u32>,
    blocks: Vec<BlockDef>,
    items: HashMap<String, ItemDef>,
    recipes: HashMap<String, RecipeDef>,
}

impl Registry {
    pub fn new() -> GameResult<Self> {
        let mut r = Registry {
            blocks_idx: HashMap::new(),
            blocks: Vec::new(),
            items: HashMap::new(),
            recipes: HashMap::new()
        };

        r.load_data(".");
        r.load_mods_data()?;

        Ok(r)
    }

    pub fn load_data(&mut self, rel_path: &str) {
        let data_dir = std::path::PathBuf::from(format!("{rel_path}/resources/data"));

        if let Err(e) = load_defs_from_dir::<BlockDef>(
            data_dir.join("blocks").as_path(),
            |block_def| self.register_block(block_def, rel_path)
        ) {
            eprintln!("Failed to load blocks: {e}");
        }

        if let Err(e) = load_defs_from_dir::<ItemDef>(
            data_dir.join("items").as_path(),
            |item_def| self.register_item(item_def, rel_path)
        ) {
            eprintln!("Failed to load items: {e}");
        }

        if let Err(e) = load_defs_from_dir::<RecipeDef>(
            data_dir.join("recipes").as_path(),
            |recipe_def| self.register_recipe(recipe_def)
        ) {
            eprintln!("Failed to load recipes: {e}");
        }
    }

    fn load_mods_data(&mut self) -> GameResult {
        let mods_dir = std::path::Path::new("./mods/");
        if !mods_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(mods_dir)? {
            let mod_path = entry?.path();
            if !mod_path.is_dir() {
                continue;
            }

            if let Some(path) = mod_path.to_str() {
                self.load_data(path);
            }
        }

        Ok(())
    }

    /// # Arguments
    /// * `def` - block definition (id, name, path to the texture, path to the script)
    /// * `rel_path` - path to the mod folder contents
    /// # Errors
    /// Returns an error if a block with this id is already registered.
    pub fn register_block(&mut self, mut def: BlockDef, rel_path: &str) -> Result<(), String> {
        if self.blocks_idx.contains_key(&def.id) {
            return Err(format!("Block id '{}' already registered", def.id));
        }

        let original = def.texture.clone();
        def.texture = format!(r"{rel_path}/{original}");

        if let Some(script) = def.script {
            let original = script;
            def.script = Some(format!(r"{rel_path}/{original}"));
        }

        let l = self.blocks_idx.len();
        self.blocks_idx.insert(def.id.clone(), u32::try_from(l).unwrap_or(u32::MAX));
        self.blocks.push(def);

        Ok(())
    }

    /// # Arguments
    /// * `def` - item definition (id, name, path to the texture)
    /// * `rel_path` - path to the mod folder contents
    /// # Errors
    /// Returns an error if an item with this id is already registered.
    pub fn register_item(&mut self, mut def: ItemDef, rel_path: &str) -> Result<(), String> {
        if self.items.contains_key(&def.id) {
            return Err(format!("Item id '{}' already registered", def.id));
        }

        let original = def.texture.clone();
        def.texture = format!(r"{rel_path}/{original}");

        self.items.insert(def.id.clone(), def);
        Ok(())
    }

    /// # Arguments
    /// * `def` - recipe definition (id, time, inputs and outputs)
    /// * `rel_path` - path to the mod folder contents
    /// # Errors
    /// Returns an error if a recipe with this id is already registered.
    pub fn register_recipe(&mut self, def: RecipeDef) -> Result<(), String> {
        if self.recipes.contains_key(&def.id) {
            return Err(format!("Reciped id '{}' already registered", def.id))
        }

        self.recipes.insert(def.id.clone(), def);
        Ok(())
    }

    /// Searches an gets a definition of a block by given `&str` id
    pub fn get_block(&self, id: &str) -> Option<&BlockDef> {
        let Some(index) = self.blocks_idx.get(id) else {
            return None;
        };

        self.blocks.get(*index as usize)
    }

    /// Returns a definition of a block directly from the `Vec<BlockDef>` by given `u32` index
    pub fn get_block_directly(&self, index: u32) -> Option<&BlockDef> {
        self.blocks.get(index as usize)
    }

    pub fn get_block_index(&self, id: &str) -> Option<u32> {
        self.blocks_idx.get(id).copied()
    }

    pub fn get_item(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }

    pub fn get_recipe(&self, id: &str) -> Option<&RecipeDef> {
        self.recipes.get(id)
    }
}

/// The global registry of all game objects (blocks, items).
pub static REGISTRY: std::sync::OnceLock<Registry> = std::sync::OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("Game registry not initialized")
}

fn load_defs_from_dir<T: serde::de::DeserializeOwned>(
    dir: &std::path::Path,
    mut register_fn: impl FnMut(T) -> Result<(), String>
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let def: T = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            register_fn(def)?;
        }
    }

    Ok(())
}

pub fn get_paths(registry: &Registry) -> Vec<String> {
    let mut texture_paths: Vec<String> = Vec::new();
    texture_paths.push(crate::MISSING_TEX.to_string());

    for block in &registry.blocks {
        texture_paths.push(block.texture.clone());
    }

    for item in registry.items.values() {
        texture_paths.push(item.texture.clone());
    }

    texture_paths.sort();
    texture_paths.dedup();

    texture_paths
}

pub fn gen_uv_cache(registry: &mut Registry, atlas: &crate::res::Atlas) -> GameResult {
    for block in registry.blocks.iter_mut() {
        block.uv = Some(*atlas.get_block_uv(block)?);
    }

    for item in registry.items.values_mut() {
        item.uv = Some(*atlas.get_item_uv(item)?)
    }

    Ok(())
}

pub fn link_scripts(registry: &Registry, script_engine: &mut crate::scripts::ScriptEngine) {
    for block in &registry.blocks {
        if let Some(script_path) = &block.script {
            if let Ok(code) = fs::read_to_string(script_path.clone()) {
                // We can call `.unwrap()` here because we're working in the registry and we know what it contains
                let id = registry.get_block_index(&block.id).unwrap();
                if let Err(e) = script_engine.load_script(id, &code) {
                    eprintln!("{e}");
                }
            } else {
                eprintln!("Script '{script_path}' not found");
            }
        }
    }
}
