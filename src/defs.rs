use std::{collections::HashMap, fs, path::Path, sync::RwLock};

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

    pub fn register_block(&mut self, def: BlockDef) -> Result<(), String> {
        if self.blocks.contains_key(&def.id) {
            return Err(format!("Block id '{}' already registered", def.id));
        }

        self.blocks.insert(def.id.clone(), def);
        Ok(())
    }

    pub fn register_item(&mut self, def: ItemDef) -> Result<(), String> {
        if self.items.contains_key(&def.id) {
            return Err(format!("Item id '{}' already registered", def.id));
        }

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

pub fn load_all_mods() {
    if let Err(e) = load_defs_from_dir::<BlockDef>(
        Path::new("data/blocks"),
        |reg, block| reg.register_block(block),
    ) {
        eprintln!("Failed to load blocks: {}", e);
    }

    if let Err(e) = load_defs_from_dir::<ItemDef>(
        Path::new("data/items"),
        |reg, item| reg.register_item(item),
    ) {
        eprintln!("Failed to load items: {}", e);
    }
}

pub fn get_block(id: &str) -> Option<BlockDef> {
    REGISTRY.read().unwrap().get_block(id).cloned()
}

pub fn get_item(id: &str) -> Option<ItemDef> {
    REGISTRY.read().unwrap().get_item(id).cloned()
}

pub fn get_paths() -> Vec<String> {
    let mut texture_paths = Vec::new();
    
    for block in REGISTRY.read().unwrap().blocks.values() {
        texture_paths.push(block.texture.clone());
    }

    for item in REGISTRY.read().unwrap().items.values() {
        texture_paths.push(item.texture.clone());
    }

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
