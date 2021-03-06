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
    /// It's in pixels, not meters! The conversion in meters is as follows:
    ///
    /// `total_walked / PIXELS_TO_METERS`
    ///
    /// Explanations: . In short, use the `get_total_walked` method.
    pub total_walked: u64,
    pub total_damages: DamageStats,
    pub max_inflicted_damage: u64,
    pub max_received_damage: u64,
    pub total_healed: u64,
    pub enemies: HashMap<CharacterKind, DamageStats>,
    pub total_quests_done: u64,
    pub total_quests_failed: u64,
    pub total_quests_abandoned: u64,
}

impl PlayerStats {
    /// Returns the distance in **centimers**!
    pub fn get_total_walked(&self) -> u64 {
        self.total_walked * 100 / crate::PIXELS_TO_METERS as u64
    }
}
