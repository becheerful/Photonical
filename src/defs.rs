use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{res::Atlas, script::ScriptEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: String,
    pub name: String,
    pub texture: String,
    pub script: Option<String>,
    #[serde(skip)]
    /// position in the dynamically stitched texture atlas
    pub uv: Option<ggez::graphics::Rect>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub texture: String,
    #[serde(skip)]
    /// position in the dynamically stitched texture atlas
    pub uv: Option<ggez::graphics::Rect>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDef {
    pub id: String,
    pub time: f32,
    pub inputs: Vec<(String, u32)>,
    pub outputs: Vec<(String, u32)>,
}

pub struct Registry {
    blocks: HashMap<String, BlockDef>,
    items: HashMap<String, ItemDef>,
    recipes: HashMap<String, RecipeDef>,
}

impl Registry {
    pub fn new() -> Self {
        let mut r = Registry {
            blocks: HashMap::new(),
            items: HashMap::new(),
            recipes: HashMap::new()
        };

        r.load_data(".");
        r.load_mods_data();

        r
    }

    pub fn load_data(&mut self, rel_path: &str) {
        let data_dir = PathBuf::from(format!("{}/resources/data", rel_path));

        if let Err(e) = load_defs_from_dir::<BlockDef>(
            data_dir.join("blocks").as_path(),
            |block_def| return self.register_block(block_def, rel_path)
        ) {
            eprintln!("Failed to load blocks: {}", e);
        }

        if let Err(e) = load_defs_from_dir::<ItemDef>(
            data_dir.join("items").as_path(),
            |item_def| return self.register_item(item_def, rel_path)
        ) {
            eprintln!("Failed to load items: {}", e);
        }

        if let Err(e) = load_defs_from_dir::<RecipeDef>(
            data_dir.join("recipes").as_path(),
            |recipe_def| return self.register_recipe(recipe_def)
        ) {
            eprintln!("Failed to load recipes: {}", e);
        }
    }

    fn load_mods_data(&mut self) {
        let mods_dir = Path::new("./mods/");
        if !mods_dir.exists() {
            return;
        }

        for entry in fs::read_dir(mods_dir).unwrap() {
            let mod_path = entry.unwrap().path();
            if !mod_path.is_dir() {
                continue;
            }

            self.load_data(mod_path.to_str().unwrap());
        }
    }

    /// # Arguments
    /// * `def` - block definition (id, name, path to the texture, path to the script)
    /// * `rel_path` - path to the mod folder contents
    /// # Errors
    /// Returns an error if a block with this id is already registered.
    pub fn register_block(&mut self, mut def: BlockDef, rel_path: &str) -> Result<(), String> {
        if self.blocks.contains_key(&def.id) {
            return Err(format!("Block id '{}' already registered", def.id));
        }

        let original = def.texture.clone();
        def.texture = format!(r"{}/{}", rel_path, original);
        
        if let Some(script) = def.script {
            let original = script;
            def.script = Some(format!(r"{}/{}", rel_path, original));
        }

        self.blocks.insert(def.id.clone(), def);
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
        def.texture = format!(r"{}/{}", rel_path, original);

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

    pub fn get_block(&self, id: &str) -> Option<&BlockDef> {
        self.blocks.get(id)
    }

    pub fn get_item(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }
    
    pub fn get_recipe(&self, id: &str) -> Option<&RecipeDef> {
        self.recipes.get(id)
    }
}

/// The global registry of all game objects (blocks, items).
pub static REGISTRY: OnceCell<Registry> = OnceCell::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("Game registry not initialized")
}

fn load_defs_from_dir<T: DeserializeOwned>(
    dir: &Path,
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

pub fn get_block(id: &str) -> Option<BlockDef> {
    registry().get_block(id).cloned()
}

pub fn get_item(id: &str) -> Option<ItemDef> {
    registry().get_item(id).cloned()
}

pub fn get_blocks() -> HashMap<String, BlockDef> {
    registry().blocks.clone()
}

pub fn get_items() -> HashMap<String, ItemDef> {
    registry().items.clone()
}

pub fn get_paths(registry: &Registry) -> Vec<String> {
    let mut texture_paths: Vec<String> = Vec::new();
    texture_paths.push(crate::MISSING_TEX.to_string());
    
    for block in registry.blocks.values() {
        texture_paths.push(block.texture.clone());
    }

    for item in registry.items.values() {
        texture_paths.push(item.texture.clone());
    }

    texture_paths.sort();
    texture_paths.dedup();

    texture_paths
}

pub fn gen_uv_cache(registry: &mut Registry, atlas: &Atlas) {
    for block in registry.blocks.values_mut() {
        block.uv = Some(*atlas.get_block_uv(block));
    }

    for item in registry.items.values_mut() {
        item.uv = Some(*atlas.get_item_uv(item))
    }
}

pub fn link_scripts(registry: &Registry, script_engine: &mut ScriptEngine) {
    for block in &registry.blocks {
        if let Some(script_path) = &block.1.script {
            if let Ok(code) = fs::read_to_string(script_path.clone()) {
                if let Err(e) = script_engine.load_script(&block.0, &code) {
                    eprintln!("{}", e);
                }
            } else {
                eprintln!("Script '{}' not found", script_path);
            }
        }
    }
}
