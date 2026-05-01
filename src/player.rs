use ggez::{glam::Vec2, input::keyboard::KeyCode};

use crate::{defs, entity::Entity};

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec2,
    pub movement_speed: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera { pos: Vec2::ZERO, movement_speed: 8.0 }
    }

    pub fn get_movement_vector(&mut self, keycode: KeyCode) -> Option<Vec2> {
        match keycode {
            KeyCode::W => Some(Vec2 { x:  0.0, y: -1.0 }),
            KeyCode::A => Some(Vec2 { x: -1.0, y:  0.0 }),
            KeyCode::S => Some(Vec2 { x:  0.0, y:  1.0 }),
            KeyCode::D => Some(Vec2 { x:  1.0, y:  0.0 }),
            _ => None,
        }
    }

    pub fn move_towards(&mut self, dir: Vec2, bounds: Vec2) {
        self.pos = (self.pos + dir * self.movement_speed)
            .max(Vec2::ZERO)
            .min(bounds);
    }
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub id: String,
    pub is_item: bool,
    pub count: u16,
}

impl ItemStack {
    pub fn new(id: String, is_item: bool, count: u16) -> Self {
        Self { id, is_item, count }
    }

    pub fn max_stack_size(&self) -> u16 {
        if self.is_item {
            if let Some(item) = defs::get_item(&self.id) {
                return item.stack_size;
            }
        }

        99
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub camera: Camera,
    pub entity: Entity,
    pub inventory: Vec<Option<ItemStack>>,
    pub current_slot: usize,
}

impl Player {
    pub fn new(max_health: u16, slot_count: usize) -> Self {
        Player {
            camera: Camera::new(),
            entity: Entity::new(max_health),
            inventory: vec![None; slot_count],
            current_slot: 0,
        }
    }

    /// Adds the specified item stack to the player's inventory.
    /// Returns a modified ItemStack of leftover item that could not be added
    /// if the inventory lacks sufficient space.
    pub fn add_item(&mut self, mut stack: ItemStack) -> Result<(), ItemStack> {
        for slot in &mut self.inventory {
            if let Some(existing) = slot {
                if existing.id == stack.id {
                    let max = existing.max_stack_size();
                    let available = max - existing.count;

                    if available > 0 {
                        let take = stack.count.min(available);
                        stack.count -= take;

                        if stack.count == 0 {
                            return Ok(());
                        }
                    }
                }
            }
        }

        for slot in &mut self.inventory {
            if slot.is_none() {
                *slot = Some(stack.clone());
                return Ok(());
            }
        }

        return Err(stack);
    }

    pub fn remove_item(&mut self, slot: usize, count: u16) -> Option<ItemStack> {
        if let Some(stack) = &mut self.inventory[slot] {
            let remove = count.min(stack.count);
            stack.count -= remove;

            if stack.count == 0 {
                let taken = self.inventory[slot].take();
                return taken;
            } else {
                return Some(ItemStack::new(stack.id.clone(), stack.is_item, remove));
            }
        }

        None
    }

    pub fn get(&self, slot: usize) -> Option<&ItemStack> {
        self.inventory[slot].as_ref()
    }

    pub fn set(&mut self, slot: usize, stack: Option<ItemStack>) {
        self.inventory[slot] = stack;
    }
}
