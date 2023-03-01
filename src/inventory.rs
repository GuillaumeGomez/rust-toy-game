use bevy::prelude::Component;

use crate::weapon::Weapon;

#[derive(Debug)]
pub enum InventoryItem {
    Weapon,
    Collectible { quantity: u16 },
}

#[derive(Debug, Component)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub gold: u32,
}
