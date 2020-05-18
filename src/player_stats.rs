use crate::character::CharacterKind;

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DamageStats {
    pub total_inflicted_damages: u64,
    pub total_received_damages: u64,
    pub total_kills: u64,
    pub total_deaths: u64,
    pub total_healed: u64,
}

#[derive(Debug, Default)]
pub struct PlayerStats {
    pub total_walked: u64,
    pub total_damages: DamageStats,
    pub max_inflicted_damage: u64,
    pub max_received_damage: u64,
    pub total_healed: u64,
    pub enemies: HashMap<CharacterKind, DamageStats>,
}
