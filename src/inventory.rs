use bevy::prelude::Component;

use crate::weapon::Weapon;

#[derive(Debug)]
pub enum InventoryItem {
    Weapon(Weapon),
    Collectible { quantity: u16 },
}

#[derive(Debug, Component)]
pub struct Inventory {
    pub items: Vec<Option<InventoryItem>>,
    pub gold: u32,
    pub equipped_weapon: Option<Weapon>,
}

impl Inventory {
    pub fn new(nb_slots: usize, gold: u32, equipped_weapon: Option<Weapon>) -> Self {
        let mut items = Vec::with_capacity(nb_slots);

        for _ in 0..nb_slots {
            items.push(None);
        }
        Self {
            items,
            gold,
            equipped_weapon,
        }
    }
}
