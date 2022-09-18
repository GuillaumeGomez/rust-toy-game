use crate::stat::Stat;
use crate::STAT_POINTS_PER_LEVEL;

use bevy::ecs::component::Component;

#[derive(Debug)]
pub struct CharacterStats {
    pub health: Stat,
    pub mana: Stat,
    pub stamina: Stat,
    pub defense: u32,
    pub attack: u32,
    // FIXME: this isn't used at the moment.
    pub attack_speed: u32,
    pub magical_attack: u32,
    pub magical_defense: u32,
    /// It also takes into account the opponent level, agility and dexterity.
    pub dodge_change: u32,
    /// It also takes into account the opponent level, agility and dexterity.
    pub critical_attack_chance: u32,
    /// How far you go in one second.
    pub move_speed: f32,
}

#[derive(Debug)]
pub struct CharacterPoints {
    pub strength: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub stamina: u32,
    pub agility: u32,
    pub dexterity: u32,
}

impl CharacterPoints {
    pub fn level_1() -> Self {
        Self {
            strength: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            stamina: 1,
            agility: 1,
            dexterity: 1,
        }
    }

    pub fn assigned_points(&self) -> u32 {
        // All fields should be listed here.
        self.strength
            + self.constitution
            + self.intelligence
            + self.wisdom
            + self.stamina
            + self.agility
            + self.dexterity
    }

    pub fn generate_stats(&self, level: u16) -> CharacterStats {
        // character points
        //
        // At each level, health and mana raise a bit
        //
        // strength -> raises attack (and health a bit, defense a bit)
        // constitution -> raises health and life regen (and attack a bit, and defense a bit)
        // intelligence -> raises mana (and magical attack/defense a bit, and mana regen a bit)
        // wisdom -> raise magical attack/defense (and mana a bit, and mana regen a bit)
        // stamina -> raises endurance (and health a bit, and defense a bit, and endurance regen a bit)
        // agility -> raises attack speed a bit x 2, dodge change a bit, critical hit a bit, move speed a bit
        // dexterity -> raises critical hit a bit x 2, attack a bit, attack speed a bit

        let level = level as u32;
        // We start with 100 HP (95 + 5 * 1).
        let total_health =
            95 + 5 * level + 10 * self.constitution + 2 * self.strength + 2 * self.stamina;
        let health_regen_speed = 1. + 1. * (self.constitution as f32);
        // We start with 20 MP (16 + 4 * 1).
        let total_mana = 16 + 4 * level + 10 * self.intelligence + 4 * self.wisdom;
        let mana_regen_speed = 0.5 + 0.5 * (self.intelligence as f32) + 0.5 * (self.wisdom as f32);
        // We start with 50 SP.
        let total_stamina = 50 + 5 * self.stamina;
        let stamina_regen_speed = 30. + 1. * (self.stamina as f32);
        CharacterStats {
            health: Stat::new(health_regen_speed, total_health as _),
            mana: Stat::new(mana_regen_speed, total_mana as _),
            stamina: Stat::new(stamina_regen_speed, total_stamina as _),
            defense: 2 * self.constitution + 1 * self.stamina,
            attack: level + 5 * self.strength + self.constitution / 2 + self.dexterity / 2,
            // FIXME: for now this is useless.
            attack_speed: 1 + 2 * self.agility + self.dexterity,
            magical_attack: level + 2 * self.wisdom + self.intelligence,
            magical_defense: level / 2 + self.wisdom + self.intelligence / 2,
            dodge_change: level + self.agility,
            critical_attack_chance: level + 2 * self.dexterity + self.agility,
            // You gain 1% of speed every four level.
            move_speed: 1. + (level as f32) / 400.,
        }
    }
}

#[derive(Component, Debug)]
pub struct Character {
    // pub action: Action,
    // pub kind: CharacterKind,
    pub xp_to_next_level: u64,
    pub xp: u64,
    pub level: u16,
    pub stats: CharacterStats,
    pub points: CharacterPoints,
    pub unused_points: u32,
    // pub texture_handler: TextureHandler,
    // pub weapon: Weapon,
    // pub is_running: bool,
    // /// How much time you need to move of 1.
    // pub speed: u32,
    // /// When "move_delay" is superior than "speed", we trigger the movement.
    // pub move_delay: u32,
    // /// How much time we show a tile before going to the next one.
    // pub tile_duration: u32,
    // /// When "tile_delay" is superior to "tile_duration", we change the tile.
    // pub tile_delay: u32,
    // /// This ID is used when this character is attacking someone else. This "someone else" will
    // /// invincible to any other attack from your ID until the total attack time is over.
    // pub id: Id,
    // pub invincible_against: Vec<InvincibleAgainst>,
    // pub statuses: Vec<Status>,
    // pub show_health_bar: bool,
    // pub death_animation: Option<Animation>,
    // /// (x, y, delay)
    // pub effect: RefCell<Option<(f32, f32, u32)>>,
    // pub weapon_action: Option<WeaponAction>,
    // pub blocking_direction: Option<Direction>,
    // pub animations: Vec<Animation>,
    // /// When moving, only the feet should be taken into account, not the head. So this is hitbox
    // /// containing width and height based on the bottom of the texture.
    // pub move_hitbox: (u32, u32),
}

fn compute_xp_to_next_level(level: u16) -> u64 {
    let mut x = 100;
    for _ in 0..level {
        x += x - x / 2;
    }
    x
}

fn compute_total_nb_points(level: u16) -> u32 {
    let mut nb_points = 0;
    for _ in 1..level {
        nb_points += STAT_POINTS_PER_LEVEL;
    }
    nb_points
}

impl Character {
    pub fn new(level: u16, xp: u64, points: CharacterPoints) -> Self {
        let stats = points.generate_stats(level);
        let unassigned = points.assigned_points();
        Self {
            xp_to_next_level: compute_xp_to_next_level(level),
            xp,
            level,
            stats,
            points,
            unused_points: unassigned + compute_total_nb_points(level),
        }
    }

    pub fn increase_xp(
        &mut self,
        xp_to_add: u64, /*, textures: &Textures<'_>, _env: Option<&mut Env>*/
    ) {
        self.xp += xp_to_add;
        if self.xp >= self.xp_to_next_level {
            self.level += 1;
            self.xp = self.xp - self.xp_to_next_level;
            self.xp_to_next_level = self.xp_to_next_level + self.xp_to_next_level / 2;
            self.reset_stats();
            self.stats = self.points.generate_stats(self.level);
            self.unused_points += STAT_POINTS_PER_LEVEL;
            // self.animations.push(Animation::new_level_up(textures));
        }
    }

    pub fn use_stat_point(&mut self) {
        // FIXME: save the new character status on disk?
        self.unused_points = self.level as u32
            * STAT_POINTS_PER_LEVEL
                .checked_sub(self.points.assigned_points())
                .unwrap_or(0);
        self.stats = self.points.generate_stats(self.level);
    }

    pub fn reset_stats(&mut self) {
        self.stats.health.reset();
        self.stats.mana.reset();
        self.stats.stamina.reset();
    }
}
