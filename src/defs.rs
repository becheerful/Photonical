use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::RwLock};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::res::Atlas;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: String,
    pub name: String,
    pub texture: String,
    #[serde(skip)]
    pub uv: Option<ggez::graphics::Rect>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub stack_size: u16,
    pub texture: String,
    #[serde(skip)]
    pub uv: Option<ggez::graphics::Rect>
}

pub struct Registry {
    blocks: HashMap<String, BlockDef>,
    items: HashMap<String, ItemDef>,
}

impl Registry {
    fn new() -> Self {
        Self { blocks: HashMap::new(), items: HashMap::new() }
    }

    /// # Arguments
    /// * `def` - block definition (id, name, path to the texture)
    /// * `rel_path` - path to the mod folder contents
    /// # Errors
    /// Returns an error if a block with this id is already registered.
    pub fn register_block(&mut self, mut def: BlockDef, rel_path: &str) -> Result<(), String> {
        if self.blocks.contains_key(&def.id) {
            return Err(format!("Block id '{}' already registered", def.id));
        }

        let original = def.texture.clone();
        def.texture = format!(r"{}{}", rel_path, original);

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
        def.texture = format!(r"{}{}", rel_path, original);

        self.items.insert(def.id.clone(), def);
        Ok(())
    }

    pub fn get_block(&self, id: &str) -> Option<&BlockDef> {
        self.blocks.get(id)
    }

    pub fn get_item(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }
}

/// The global registry of all game objects (blocks, items).
pub static REGISTRY: Lazy<RwLock<Registry>> = Lazy::new(|| RwLock::new(Registry::new()));

fn load_defs_from_dir<T: DeserializeOwned>(
    dir: &Path,
    mut register_fn: impl FnMut(&mut Registry, T) -> Result<(), String>
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
            let mut registry = REGISTRY.write().unwrap();
            register_fn(&mut registry, def)?;
        }
    }

    Ok(())
}

pub fn load_base_data() {
    if let Err(e) = load_defs_from_dir::<BlockDef>(
        Path::new("./resources/data/blocks"),
        |reg, block_def| return reg.register_block(block_def, ".")
    ) {
        eprintln!("Failed to load blocks: {}", e);
    }

    if let Err(e) = load_defs_from_dir::<ItemDef>(
        Path::new("./resources/data/items"),
        |reg, item_def| return reg.register_item(item_def, ".")
    ) {
        eprintln!("Failed to load items: {}", e);
    }
}

fn parse_mod(path: &PathBuf) {
    let data_dir = path.join("resources/data");
    // let scripts_dir = path.join("src");

    let blocks_dir = data_dir.join("blocks");
    if blocks_dir.exists() {
        if let Err(e) = load_defs_from_dir::<BlockDef>(
            blocks_dir.as_path(),
            |reg, block_def| reg.register_block(block_def, path.to_str().unwrap())
        ) {
            eprintln!("Failed to load items from mod: {}", e);
        }
    }

    let items_dir = data_dir.join("items");
    if items_dir.exists() {
        if let Err(e) = load_defs_from_dir::<ItemDef>(
            items_dir.as_path(),
            |reg, item_def| reg.register_item(item_def, path.to_str().unwrap())
        ) {
            eprintln!("Failed to load blocks from mod: {}", e);
        }
    }
}

pub fn load_mods_data() {
    let mods_dir = Path::new("./mods/");
    if !mods_dir.exists() {
        return;
    }

    for entry in fs::read_dir(mods_dir).unwrap() {
        let mod_path = entry.unwrap().path();
        if !mod_path.is_dir() {
            continue;
        }

        parse_mod(&mod_path);
    }
}

pub fn get_block(id: &str) -> Option<BlockDef> {
    REGISTRY.read().unwrap().get_block(id).cloned()
}

pub fn get_item(id: &str) -> Option<ItemDef> {
    REGISTRY.read().unwrap().get_item(id).cloned()
}

pub fn get_paths() -> Vec<String> {
    let mut texture_paths: Vec<String> = Vec::new();
    texture_paths.push(crate::MISSING_TEX.to_string());
    
    for block in REGISTRY.read().unwrap().blocks.values() {
        texture_paths.push(block.texture.clone());
    }

    for item in REGISTRY.read().unwrap().items.values() {
        texture_paths.push(item.texture.clone());
    }

    texture_paths.sort();
    texture_paths.dedup();

    texture_paths
}

pub fn gen_uv_cache(atlas: &Atlas) {
    for block in REGISTRY.write().unwrap().blocks.values_mut() {
        block.uv = Some(*atlas.get_block_uv(block));
    }

    for item in REGISTRY.write().unwrap().items.values_mut() {
        item.uv = Some(*atlas.get_item_uv(item))
    }
}
