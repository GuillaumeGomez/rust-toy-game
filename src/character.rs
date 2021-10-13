use crate::sdl2::pixels::Color;
use crate::sdl2::rect::Rect;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::animation::Animation;
use crate::enemies::Skeleton;
use crate::enemy::Enemy;
use crate::env::Env;
use crate::map::Map;
use crate::player::Player;
use crate::reward::RewardInfo;
use crate::stat::Stat;
use crate::status::Status;
use crate::system::System;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::texture_holder::{TextureHolder, Textures};
use crate::weapon::Weapon;
// use crate::window::UpdateKind;
use crate::{GetDimension, GetPos, Id, MAP_CASE_SIZE, MAP_SIZE, ONE_SECOND};

const STAT_POINTS_PER_LEVEL: u32 = 3;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharacterKind {
    Player,
    Enemy, // TODO: Enemy is just temporary, it'll be replaced by an id for each kind of monsters
}

impl CharacterKind {
    fn is_player(self) -> bool {
        self == Self::Player
    }
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[repr(usize)]
pub enum Direction {
    Down = 0,
    Up = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    pub fn is_right(&self) -> bool {
        match *self {
            Self::Right => true,
            _ => false,
        }
    }
    pub fn is_left(&self) -> bool {
        match *self {
            Self::Left => true,
            _ => false,
        }
    }
    pub fn is_up(&self) -> bool {
        match *self {
            Self::Up => true,
            _ => false,
        }
    }
    pub fn is_down(&self) -> bool {
        match *self {
            Self::Down => true,
            _ => false,
        }
    }
    pub fn get_opposite(&self) -> Direction {
        match *self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
    pub fn is_opposite(&self, other: Direction) -> bool {
        self.get_opposite() == other
    }
    pub fn is_adjacent(&self, other: Direction) -> bool {
        match *self {
            Self::Up | Self::Down => other.is_left() || other.is_right(),
            Self::Right | Self::Left => other.is_up() || other.is_down(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct Action {
    pub direction: Direction,
    pub secondary: Option<Direction>,
    pub movement: Option<u64>,
}

impl Action {
    /// Returns `(x, y, width, height, draw_width, draw_height)`.
    pub fn compute_current(
        &self,
        is_running: bool,
        textures: &TextureHandler,
    ) -> (i32, i32, u32, u32, u32, u32) {
        if let Some(ref pos) = self.movement {
            let (info, nb_animations) = &textures.actions_moving[self.direction as usize];
            let pos = if is_running {
                (pos % 30) as i32 / (30 / nb_animations)
            } else {
                (pos % 60) as i32 / (60 / nb_animations)
            };
            if let Some((tile_width, tile_height)) = textures.forced_size {
                (
                    pos * info.incr_to_next + info.x,
                    info.y,
                    info.width(),
                    info.height(),
                    tile_width,
                    tile_height,
                )
            } else {
                (
                    pos * info.incr_to_next + info.x,
                    info.y,
                    info.width(),
                    info.height(),
                    info.width(),
                    info.height(),
                )
            }
        } else {
            let info = &textures.actions_standing[self.direction as usize];
            if let Some((tile_width, tile_height)) = textures.forced_size {
                (
                    info.x,
                    info.y,
                    info.width(),
                    info.height(),
                    tile_width,
                    tile_height,
                )
            } else {
                (
                    info.x,
                    info.y,
                    info.width(),
                    info.height(),
                    info.width(),
                    info.height(),
                )
            }
        }
    }

    pub fn get_dimension(&self, textures: &TextureHandler) -> Dimension {
        let mut dim;
        if let Some(_) = self.movement {
            dim = textures.actions_moving[self.direction as usize].0.clone();
            if let Some((tile_width, tile_height)) = textures.forced_size {
                dim.set_width(tile_width);
                dim.set_height(tile_height);
            }
        } else {
            dim = textures.actions_standing[self.direction as usize].clone();
            if let Some((tile_width, tile_height)) = textures.forced_size {
                dim.set_width(tile_width);
                dim.set_height(tile_height);
            }
        }
        dim
    }

    pub fn get_specific_dimension(&self, textures: &TextureHandler, dir: Direction) -> Dimension {
        let mut dim = textures.actions_moving[dir as usize].0.clone();
        if let Some((tile_width, tile_height)) = textures.forced_size {
            dim.set_width(tile_width);
            dim.set_height(tile_height);
        }
        dim
    }
}

pub struct InvincibleAgainst {
    id: Id,
    remaining_time: u64,
}

impl InvincibleAgainst {
    fn new(id: Id, remaining_time: u64) -> InvincibleAgainst {
        InvincibleAgainst { id, remaining_time }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Obstacle {
    Map,
    Character,
    None,
}

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
    // FIXME: speed should be here!
}

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
    fn assigned_points(&self) -> u32 {
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
        }
    }
}

pub struct Character {
    pub action: Action,
    pub x: i64,
    pub y: i64,
    pub kind: CharacterKind,
    pub xp_to_next_level: u64,
    pub xp: u64,
    pub level: u16,
    pub stats: CharacterStats,
    pub points: CharacterPoints,
    pub unused_points: u32,
    pub texture_handler: TextureHandler,
    pub weapon: Option<Weapon>,
    pub is_running: bool,
    /// How much time you need to move of 1.
    pub speed: u64,
    /// When "move_delay" is superior than "speed", we trigger the movement.
    pub move_delay: u64,
    /// This ID is used when this character is attacking someone else. This "someone else" will
    /// invincible to any other attack from your ID until the total attack time is over.
    pub id: Id,
    pub invincible_against: Vec<InvincibleAgainst>,
    pub statuses: Vec<Status>,
    pub show_health_bar: bool,
    pub death_animation: Option<Animation>,
    /// (x, y, delay)
    pub effect: RefCell<Option<(i64, i64, u64)>>,
    pub animations: Vec<Animation>,
    /// When moving, only the feet should be taken into account, not the head. So this is hitbox
    /// containing width and height based on the bottom of the texture.
    pub move_hitbox: (u32, u32),
}

impl Character {
    pub fn increase_xp(&mut self, xp_to_add: u64, textures: &Textures<'_>, env: Option<&mut Env>) {
        self.xp += xp_to_add;
        if self.xp >= self.xp_to_next_level {
            self.level += 1;
            self.xp = self.xp - self.xp_to_next_level;
            self.xp_to_next_level = self.xp_to_next_level + self.xp_to_next_level / 2;
            self.reset_stats();
            self.stats = self.points.generate_stats(self.level);
            self.unused_points += STAT_POINTS_PER_LEVEL;
            self.animations.push(Animation::new_level_up(textures));
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

    pub fn check_hitbox(
        &self,
        new_x: i64,
        new_y: i64,
        map_data: &[u8],
        dir_to_check: Direction,
    ) -> bool {
        let dimension = self
            .action
            .get_specific_dimension(&self.texture_handler, dir_to_check);
        let new_x = new_x + (dimension.width() / 2 - self.move_hitbox.0 / 2) as i64;
        let new_y = new_y + (dimension.height() - self.move_hitbox.1) as i64;
        let initial_x = new_x / MAP_CASE_SIZE;
        let initial_y = new_y / MAP_CASE_SIZE;
        let width = ::std::cmp::max(1, self.move_hitbox.0 / MAP_CASE_SIZE as u32) as i64;
        let height = ::std::cmp::max(1, self.move_hitbox.1 / MAP_CASE_SIZE as u32) as i64;

        match dir_to_check {
            Direction::Down => {
                let y = (height + initial_y) * MAP_SIZE as i64;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix as i64;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Up => {
                let y = initial_y * MAP_SIZE as i64;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix as i64;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Right => {
                for iy in 0..height {
                    let map_pos = (initial_y + iy as i64) * MAP_SIZE as i64 + initial_x + width + 1;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Left => {
                for iy in 0..height {
                    let map_pos = (initial_y + iy as i64) * MAP_SIZE as i64 + initial_x - 1;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn check_character_move(&self, x: i64, y: i64, character: &Character) -> bool {
        if character.id == self.id {
            return true;
        }
        let self_width = self.move_hitbox.0;
        let self_height = self.move_hitbox.1;
        let self_x = x + (self.width() / 2 - self_width / 2) as i64;
        let self_y = y + (self.height() - self_height) as i64;
        let other_width = character.move_hitbox.0;
        let other_height = character.move_hitbox.1;
        let other_x = character.x() + (character.width() / 2 - other_width / 2) as i64;
        let other_y = character.y() + (character.height() - other_height) as i64;

        !(self_x + self_width as i64 >= other_x
            && self_x <= other_x + other_width as i64
            && self_y + self_height as i64 >= other_y
            && self_y <= other_y + other_height as i64)
    }

    pub fn check_map_pos(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<Skeleton>],
        new_x: i64,
        new_y: i64,
        ignore_id: Option<Id>,
    ) -> Obstacle {
        let initial_x =
            (new_x + (self.width() / 2 - self.move_hitbox.0 / 2) as i64 - map.x) / MAP_CASE_SIZE;
        let initial_y =
            (new_y + (self.height() - self.move_hitbox.1) as i64 - map.y) / MAP_CASE_SIZE;
        let width = (self.move_hitbox.0 / MAP_CASE_SIZE as u32) as i64;
        let height = (self.move_hitbox.1 / MAP_CASE_SIZE as u32) as i64;

        for y in 0..height {
            let y = (y + initial_y) * MAP_SIZE as i64;
            for x in 0..width {
                let map_pos = y + initial_x + x;
                if map_pos < 0 || map.data.get(map_pos as usize).unwrap_or(&1) != &0 {
                    return Obstacle::Map;
                }
            }
        }
        let ignore_id = ignore_id.unwrap_or(self.id);
        if npcs
            .iter()
            .all(|n| n.id() == ignore_id || self.check_character_move(new_x, new_y, n.character()))
            && players
                .iter()
                .all(|p| p.id == ignore_id || self.check_character_move(new_x, new_y, &p))
        {
            Obstacle::None
        } else {
            Obstacle::Character
        }
    }

    /// `x_add` and `y_add` are used in case you want to move in two directions at once, so when
    /// checking the second direction, you actually already "moved" and don't check a bad position.
    pub fn check_move(
        &self,
        direction: Direction,
        map: &Map,
        players: &[Player],
        npcs: &[Box<Skeleton>],
        // In case of a move on two axes, we have to provide the result of the first move too!
        x_add: i64,
        y_add: i64,
    ) -> (i64, i64) {
        let info = self
            .action
            .get_specific_dimension(&self.texture_handler, direction);
        let x = self.x() + x_add;
        let y = self.y() + y_add;
        let (x_add, y_add) = match direction {
            Direction::Down
                if info.height() as i64 + y + 1 < map.y + MAP_SIZE as i64 * MAP_CASE_SIZE =>
            {
                (0, 1)
            }
            Direction::Up if y - 1 >= map.y => (0, -1),
            Direction::Left if x - 1 >= map.x => (-1, 0),
            Direction::Right
                if info.width() as i64 + x + 1 < map.x + MAP_SIZE as i64 * MAP_CASE_SIZE =>
            {
                (1, 0)
            }
            _ => return (0, 0),
        };

        fn call(
            self_id: Id,
            c: &Character,
            move_hitbox: &(u32, u32),
            x: i64,
            y: i64,
            width: u32,
            height: u32,
            direction: Direction,
        ) -> bool {
            if self_id == c.id {
                return true;
            }
            let self_x = x + (width / 2 - move_hitbox.0 / 2) as i64;
            let self_y = y + (height - move_hitbox.1) as i64;
            let other_width = c.move_hitbox.0;
            let other_height = c.move_hitbox.1;
            let other_x = c.x() + (c.width() / 2 - other_width / 2) as i64;
            let other_y = c.y() + (c.height() - other_height) as i64;

            let width = move_hitbox.0 as i64;
            let height = move_hitbox.1 as i64;

            !match direction {
                Direction::Down => {
                    self_x + width >= other_x
                        && self_x <= other_x + other_width as i64
                        && self_y + height + 1 >= other_y
                        && self_y + height <= other_y + other_height as i64
                }
                Direction::Up => {
                    self_x + width >= other_x
                        && self_x <= other_x + other_width as i64
                        && self_y >= other_y
                        && self_y - 1 <= other_y + other_height as i64
                }
                Direction::Right => {
                    self_x + width + 1 >= other_x
                        && self_x + width <= other_x + other_width as i64
                        && self_y + height >= other_y
                        && self_y <= other_y + other_height as i64
                }
                Direction::Left => {
                    self_x >= other_x
                        && self_x - 1 <= other_x + other_width as i64
                        && self_y + height >= other_y
                        && self_y <= other_y + other_height as i64
                }
            }
        }

        let self_x = x + x_add;
        let self_y = y + y_add;
        // NPC moves are a bit more restricted than players'.
        if if self.kind.is_player() {
            let width = self.width();
            let height = self.height();
            self.check_hitbox(self_x - map.x, self_y - map.y, &map.data, direction)
                && npcs.iter().all(|n| {
                    call(
                        self.id,
                        n.character(),
                        &self.move_hitbox,
                        self_x,
                        self_y,
                        width,
                        height,
                        direction,
                    )
                })
                && players.iter().all(|p| {
                    call(
                        self.id,
                        &p,
                        &self.move_hitbox,
                        self_x,
                        self_y,
                        width,
                        height,
                        direction,
                    )
                })
        } else {
            self.check_hitbox(self_x - map.x, self_y - map.y, &map.data, direction)
                && npcs
                    .iter()
                    .all(|n| self.check_character_move(self_x, self_y, n.character()))
                && players
                    .iter()
                    .all(|n| self.check_character_move(self_x, self_y, &n))
        } {
            (x_add, y_add)
        } else {
            (0, 0)
        }
    }

    /// `x_add` and `y_add` are used in case you want to move in two directions at once, so when
    /// checking the second direction, you actually already "moved" and don't check a bad position.
    pub fn inner_check_move(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<Skeleton>],
        primary_direction: Direction,
        secondary_direction: Option<Direction>,
        x_add: i64,
        y_add: i64,
    ) -> (i64, i64) {
        let (mut x, mut y) = self.check_move(primary_direction, map, players, npcs, x_add, y_add);
        if let Some(secondary_direction) = secondary_direction {
            let (x2, y2) = self.check_move(
                secondary_direction,
                map,
                players,
                npcs,
                x_add + x,
                y_add + y,
            );
            x += x2;
            y += y2;
        }
        (x, y)
    }

    /// `x_add` and `y_add` are used in case you want to move in two directions at once, so when
    /// checking the second direction, you actually already "moved" and don't check a bad position.
    pub fn inner_apply_move(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<Skeleton>],
        x_add: i64,
        y_add: i64,
    ) -> (i64, i64) {
        if self.action.movement.is_none() {
            (0, 0)
        } else {
            self.inner_check_move(
                map,
                players,
                npcs,
                self.action.direction,
                self.action.secondary,
                x_add,
                y_add,
            )
        }
    }

    pub fn draw(&mut self, system: &mut System, debug: bool) {
        let (tile_x, tile_y, tile_width, tile_height, draw_width, draw_height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        if self.is_dead() {
            if let Some(ref death) = self.death_animation {
                death.draw(
                    system,
                    self.x + draw_width as i64 / 2,
                    self.y + draw_height as i64 / 2,
                );
                return;
            }
        }
        let x = (self.x - system.x()) as i32;
        let y = (self.y - system.y()) as i32;
        if x + draw_width as i32 >= 0
            && x < system.width()
            && y + draw_height as i32 >= 0
            && y < system.height()
        {
            system.copy_to_canvas(
                self.texture_handler.texture,
                Rect::new(tile_x, tile_y, tile_width, tile_height),
                Rect::new(x, y, draw_width, draw_height),
            );
        }
        for animation in self.animations.iter() {
            animation.draw(
                system,
                self.x + draw_width as i64 / 2,
                self.y + (draw_height - animation.sprite_display_height / 2) as i64,
            );
        }
        if debug {
            system
                .canvas
                .draw_rect(Rect::new(x, y, draw_width, draw_height))
                .unwrap();
            system
                .canvas
                .draw_rect(Rect::new(
                    x + (self.width() / 2 - self.move_hitbox.0 / 2) as i32,
                    y + (self.height() - self.move_hitbox.1) as i32,
                    self.move_hitbox.0,
                    self.move_hitbox.1,
                ))
                .unwrap();
        }
        if let Some(ref weapon) = self.weapon {
            // if let Some(matrix) = weapon.compute_angle() {
            //     for (x, y) in matrix.iter() {
            //         canvas.fill_rect(Rect::new(x - screen.x, y - screen.y, 8, 8));
            //     }
            // }
            weapon.draw(system, self.id == 1);
        }

        if self.show_health_bar && !self.stats.health.is_full() {
            system.health_bar.draw(
                self.x + (draw_width as i32 - system.health_bar.width as i32) as i64 / 2,
                self.y - (system.health_bar.height + 2) as i64,
                self.stats.health.pourcent(),
                system,
            );
        }

        let x = self.x + self.width() as i64 / 2;
        for it in (0..self.statuses.len()).rev() {
            self.statuses[it].draw(system, x, self.y);
            if self.statuses[it].should_be_removed() {
                self.statuses.remove(it);
            }
        }
    }

    // TODO: add stamina consumption when attacking, depending on the weight of the weapon!
    pub fn attack(&mut self) {
        let remaining_stamina = self.stats.stamina.value();
        let to_subtract = match self.weapon {
            Some(ref mut weapon) if remaining_stamina >= weapon.weight() as u64 => {
                weapon.use_it(self.action.direction);
                weapon.weight() as u64
            }
            _ => 0,
        };
        if to_subtract != 0 {
            self.set_weapon_pos();
            self.stats.stamina.subtract(to_subtract);
        }
    }

    pub fn is_attacking(&self) -> bool {
        self.weapon
            .as_ref()
            .map(|w| w.is_attacking())
            .unwrap_or(false)
    }

    pub fn stop_attack(&mut self) {
        if let Some(ref mut weapon) = self.weapon {
            weapon.stop_use();
        }
    }

    pub fn apply_move(
        &self,
        map: &Map,
        elapsed: u64,
        players: &[Player],
        npcs: &[Box<Skeleton>],
    ) -> (i64, i64) {
        if self.is_dead() {
            return (0, 0);
        }
        let mut tmp = self.move_delay + elapsed;
        let mut stamina = self.stats.stamina.clone();
        let mut x = 0;
        let mut y = 0;

        if let Some(ref mut effect) = &mut *self.effect.borrow_mut() {
            while tmp > effect.2 && (effect.0 != 0 || effect.1 != 0) {
                if effect.0 != 0 {
                    let (x1, _) = self.check_move(
                        if effect.0 < 0 {
                            Direction::Left
                        } else {
                            Direction::Right
                        },
                        map,
                        players,
                        npcs,
                        x,
                        y,
                    );
                    if x1 != 0 {
                        x += x1;
                        effect.0 += x1 * -1;
                    } else {
                        effect.0 = 0;
                        effect.1 = 0;
                        break;
                    }
                }
                if effect.1 != 0 {
                    let (_, y1) = self.check_move(
                        if effect.1 < 0 {
                            Direction::Up
                        } else {
                            Direction::Down
                        },
                        map,
                        players,
                        npcs,
                        x,
                        y,
                    );
                    if y1 != 0 {
                        y += y1;
                        effect.1 += y1 * -1;
                    } else {
                        effect.0 = 0;
                        effect.1 = 0;
                        break;
                    }
                }
                tmp -= effect.2;
            }
        } else {
            while tmp > self.speed {
                let (x1, y1) = self.inner_apply_move(map, players, npcs, x, y);
                x += x1;
                y += y1;
                if x1 != 0 || y1 != 0 {
                    if self.is_running {
                        if stamina.value() > 0 {
                            let (x2, y2) = self.inner_apply_move(map, players, npcs, x, y);
                            x += x2;
                            y += y2;
                            stamina.subtract(1);
                        }
                    }
                } else {
                    // It means the character couldn't move in any of the direction it wanted so no
                    // need to continue this loop.
                    break;
                }
                tmp -= self.speed;
            }
        }
        (x, y)
    }

    pub fn update(&mut self, elapsed: u64, x: i64, y: i64, env: Option<&mut Env>) {
        if self.is_dead() {
            if let Some(ref mut death) = self.death_animation {
                death.update(elapsed);
            }
            return;
        }
        self.x += x;
        self.y += y;
        // Since we might change direction, better update weapon in any case...
        self.set_weapon_pos();

        let env_stamina = self.stats.stamina.refresh(elapsed);
        let env_health = self.stats.health.refresh(elapsed);
        let env_mana = self.stats.mana.refresh(elapsed);
        // if let Some(env) = env {
        //     if env_stamina {
        //         env.add_character_update("Stamina", UpdateKind::Value(self.stamina.value()));
        //     }
        //     if env_health {
        //         env.add_character_update("Health", UpdateKind::Value(self.health.value()));
        //     }
        //     if env_mana {
        //         env.add_character_update("Mana", UpdateKind::Value(self.mana.value()));
        //     }
        // }
        self.move_delay += elapsed;
        let effect = self.effect.borrow_mut().take();
        if let Some(effect) = effect {
            if effect.0 != 0 || effect.1 != 0 {
                *self.effect.borrow_mut() = Some(effect);
            }
        } else {
            while self.move_delay > self.speed {
                // We now update the animation!
                if let Some(ref mut pos) = self.action.movement {
                    *pos += 1;
                }
                let stamina_value = self.stats.stamina.value();
                if self.is_running && stamina_value > 0 {
                    self.stats.stamina.subtract(1);
                    self.is_running = stamina_value - 1 > 0;
                }
                self.move_delay -= self.speed;
            }

            if let Some(ref mut weapon) = self.weapon {
                weapon.update(elapsed);
            }
        }

        // We update the statuses display
        for status in self.statuses.iter_mut() {
            status.update(elapsed);
        }
        for pos in (0..self.animations.len()).rev() {
            self.animations[pos].update(elapsed);
            if self.animations[pos].is_done() {
                self.animations.remove(pos);
            }
        }

        // The "combat" part: we update the list of characters that can't hit this one.
        if self.invincible_against.is_empty() {
            return;
        }
        let mut i = 0;
        while i < self.invincible_against.len() {
            if self.invincible_against[i].remaining_time <= elapsed {
                self.invincible_against.remove(i);
            } else {
                self.invincible_against[i].remaining_time -= elapsed;
                i += 1;
            }
        }
    }

    fn set_weapon_pos(&mut self) {
        if let Some(ref mut weapon) = self.weapon {
            if weapon.is_blocking() {
                // To set the direction of the blocking.
                weapon.block(self.action.direction);

                let (_, _, _, _, draw_width, draw_height) = self
                    .action
                    .compute_current(self.is_running, &self.texture_handler);
                let draw_width = draw_width as i64;
                let draw_height = draw_height as i64;
                let width = weapon.width() as i64;
                let height = weapon.height() as i64;
                let (x, y) = match self.action.direction {
                    Direction::Up => (self.x + width + 2, self.y - height + 3),
                    Direction::Down => (self.x + width / 2, self.y + draw_height - 2),
                    Direction::Left => (self.x - width - 4, self.y),
                    Direction::Right => (self.x + draw_width, self.y),
                };
                weapon.set_pos(x, y);
            } else {
                let (_, _, _, _, draw_width, draw_height) = self
                    .action
                    .compute_current(self.is_running, &self.texture_handler);
                let draw_width = draw_width as i64;
                let draw_height = draw_height as i64;
                let width = weapon.width() as i64;
                let height = weapon.height() as i64;
                let (x, y) = match self.action.direction {
                    Direction::Up => (self.x + draw_width / 2 - 3, self.y - height),
                    Direction::Down => (self.x + draw_width / 2 - 4, self.y + draw_height - height),
                    Direction::Left => (self.x - 2, self.y + draw_height / 2 - height + 2),
                    Direction::Right => (
                        self.x + draw_width - width + 2,
                        self.y + draw_height / 2 - height,
                    ),
                };
                weapon.set_pos(x, y);
            }
        }
    }

    pub fn check_intersection(
        &mut self,
        attacker_id: Id,
        attacker_direction: Direction,
        weapon: &Weapon,
        matrix: &mut Option<Vec<(i64, i64)>>,
        textures: &Textures<'_>,
    ) -> i32 {
        if self.is_dead()
            || attacker_id == self.id
            || self.invincible_against.iter().any(|e| e.id == attacker_id)
        {
            return 0;
        }
        let (_tile_x, _tile_y, _, _, width, height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        let w_biggest = ::std::cmp::max(weapon.height(), weapon.width()) as i64;
        let weapon_x = if attacker_direction == Direction::Right {
            weapon.x + w_biggest
        } else {
            weapon.x
        };
        let weapon_y = if attacker_direction == Direction::Down {
            weapon.y + w_biggest
        } else {
            weapon.y
        };

        if weapon_x + w_biggest < self.x
            || weapon_x - w_biggest > self.x + width as i64
            || weapon_y + w_biggest < self.y
            || weapon_y - w_biggest > self.y + height as i64
        {
            // The weapon is too far from this character, no need to check further!
            return 0;
        }

        if matrix.is_none() {
            *matrix = weapon.compute_angle(textures);
        }
        if let Some(ref matrix) = matrix {
            if self.texture_handler.check_intersection(
                textures,
                &matrix,
                self.action.direction,
                self.action.movement.is_some(),
                (self.x, self.y),
            ) {
                // TODO: add element effects on attacks.
                // TODO2: if you attack with fire effect on a fire monster, it heals it!
                let attack = if weapon.attack >= 0 {
                    let attack = if self.is_blocking() {
                        let dir = self.get_direction();
                        {
                            let mut effect = self.effect.borrow_mut();
                            if effect.is_none() {
                                // We want the character to be moved by 6 cases.
                                let distance = MAP_CASE_SIZE * 6;
                                // We want the "animation" to last for half a second.
                                let dur = ONE_SECOND / 2 / distance as u64;
                                *effect = Some(match dir {
                                    Direction::Up => (0, distance, dur),
                                    Direction::Down => (0, distance * -1, dur),
                                    Direction::Right => (distance * -1, 0, dur),
                                    Direction::Left => (distance, 0, dur),
                                });
                            }
                        }
                        if dir.is_opposite(attacker_direction) {
                            // They're facing each other, full block on the attack!
                            weapon.attack / 2
                        } else if dir.is_adjacent(attacker_direction) {
                            // Partially blocked, only 25% of the attack is removed
                            weapon.attack * 3 / 4
                        } else {
                            // The attack is on the back, full damage!
                            weapon.attack
                        }
                    } else {
                        weapon.attack
                    };
                    let attack = if attack == 0 { 1 } else { attack };
                    self.stats.health.subtract(attack as u64);
                    attack
                } else {
                    self.stats.health.add((weapon.attack * -1) as u64);
                    weapon.attack
                };
                // TODO: not the same display if attack is negative (meaning you gain back health!).
                if !self.stats.health.is_empty() {
                    self.invincible_against
                        .push(InvincibleAgainst::new(attacker_id, weapon.total_time));
                    // TODO: add defense on characters and make computation here (also add dodge
                    // computation and the other stuff...)
                    self.statuses
                        .push(Status::new(attack.to_string(), Color::RGB(255, 0, 0)));
                }
                return weapon.attack;
            }
        }
        0
    }

    pub fn is_dead(&self) -> bool {
        self.stats.health.is_empty()
    }

    pub fn get_reward(&self) -> Option<RewardInfo> {
        Some(RewardInfo { gold: 1 })
    }

    pub fn should_be_removed(&self) -> bool {
        if !self.is_dead() {
            return false;
        }
        match self.death_animation {
            Some(ref death) => death.is_done(),
            None => true,
        }
    }

    pub fn is_blocking(&self) -> bool {
        self.weapon
            .as_ref()
            .map(|w| w.is_blocking())
            .unwrap_or(false)
    }
    pub fn block(&mut self) {
        let dir = self.get_direction();
        if if let Some(ref mut weapon) = self.weapon {
            weapon.block(dir);
            true
        } else {
            false
        } {
            self.set_weapon_pos();
        }
    }
    pub fn stop_block(&mut self) {
        if let Some(ref mut weapon) = self.weapon {
            weapon.stop_block();
        }
    }

    pub fn get_direction(&self) -> Direction {
        self.action.direction
    }

    pub fn reset_stats(&mut self) {
        self.stats.health.reset();
        self.stats.mana.reset();
        self.stats.stamina.reset();
    }

    pub fn resurrect(&mut self) {
        if !self.is_dead() {
            return;
        }
        self.reset_stats();
        // When you get resurrected "by yourself", you lose 10% of your xp.
        let tenth = self.xp_to_next_level / 10;
        if tenth <= self.xp {
            self.xp -= tenth;
        } else {
            self.xp = 0;
        }
        // TODO: also reset its position
    }
}

impl GetDimension for Character {
    fn width(&self) -> u32 {
        self.action.get_dimension(&self.texture_handler).width()
    }

    fn height(&self) -> u32 {
        self.action.get_dimension(&self.texture_handler).height()
    }
}

impl GetPos for Character {
    fn x(&self) -> i64 {
        self.x
    }

    fn y(&self) -> i64 {
        self.y
    }
}
