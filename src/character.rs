use crate::sdl2::pixels::Color;
use crate::sdl2::rect::Rect;

use std::cell::RefCell;

use crate::animation::Animation;
use crate::enemy::Enemy;
use crate::env::Env;
use crate::map::Map;
use crate::player::Player;
use crate::reward::RewardInfo;
use crate::stat::Stat;
use crate::status::Status;
use crate::system::System;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::texture_holder::Textures;
use crate::weapon::{Weapon, WeaponAction};
// use crate::window::UpdateKind;
use crate::{GetDimension, GetPos, Id, MAP_CASE_SIZE, MAP_SIZE, MAP_SIZE_WITH_CASE, ONE_SECOND};

use parry2d::shape::ConvexPolygon;

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
        matches!(self, Self::Right)
    }
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }
    pub fn is_up(&self) -> bool {
        matches!(self, Self::Up)
    }
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down)
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

#[derive(Clone, Copy, Debug)]
pub struct DirectionAndStrength {
    pub direction: Direction,
    pub strength: f32, // Very useful with joysticks!
}

impl DirectionAndStrength {
    pub fn new_with_strength(direction: Direction, strength: f32) -> Self {
        Self {
            direction,
            strength,
        }
    }
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            strength: 1.,
        }
    }
    pub fn convert_to_x_y(self) -> (f32, f32) {
        match self.direction {
            Direction::Up => (0., -self.strength),
            Direction::Down => (0., self.strength),
            Direction::Left => (-self.strength, 0.),
            Direction::Right => (self.strength, 0.),
        }
    }
}
impl std::ops::Deref for DirectionAndStrength {
    type Target = Direction;

    fn deref(&self) -> &Self::Target {
        &self.direction
    }
}
impl std::hash::Hash for DirectionAndStrength {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.direction.hash(state);
    }
}

impl PartialEq for DirectionAndStrength {
    fn eq(&self, other: &Self) -> bool {
        self.direction == other.direction
    }
}
impl PartialEq<Direction> for DirectionAndStrength {
    fn eq(&self, other: &Direction) -> bool {
        self.direction == *other
    }
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct Action {
    pub direction: DirectionAndStrength,
    pub secondary: Option<DirectionAndStrength>,
    pub movement: Option<u32>,
}

impl Action {
    /// Returns `(x, y, width, height, draw_width, draw_height)`.
    pub fn compute_current(&self, textures: &TextureHandler) -> (i32, i32, u32, u32, u32, u32) {
        if let Some(ref pos) = self.movement {
            let (info, nb_animations) = &textures.actions_moving[*self.direction as usize];
            // TODO: It shouldn't be needed but just in case...
            let pos = *pos as i32 % nb_animations;
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
            let info = &textures.actions_standing[*self.direction as usize];
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
            dim = textures.actions_moving[*self.direction as usize].0.clone();
            if let Some((tile_width, tile_height)) = textures.forced_size {
                dim.set_width(tile_width);
                dim.set_height(tile_height);
            }
        } else {
            dim = textures.actions_standing[*self.direction as usize].clone();
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
    remaining_time: u32,
}

impl InvincibleAgainst {
    fn new(id: Id, remaining_time: u32) -> InvincibleAgainst {
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
    /// How far you go in one second.
    pub move_speed: f32,
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
            // You gain 1% of speed every four level.
            move_speed: 1. + (level as f32) / 400.,
        }
    }
}

pub struct Character {
    pub action: Action,
    pub x: f32,
    pub y: f32,
    pub kind: CharacterKind,
    pub xp_to_next_level: u64,
    pub xp: u64,
    pub level: u16,
    pub stats: CharacterStats,
    pub points: CharacterPoints,
    pub unused_points: u32,
    pub texture_handler: TextureHandler,
    pub weapon: Weapon,
    pub is_running: bool,
    /// How much time you need to move of 1.
    pub speed: u32,
    /// When "move_delay" is superior than "speed", we trigger the movement.
    pub move_delay: u32,
    /// How much time we show a tile before going to the next one.
    pub tile_duration: u32,
    /// When "tile_delay" is superior to "tile_duration", we change the tile.
    pub tile_delay: u32,
    /// This ID is used when this character is attacking someone else. This "someone else" will
    /// invincible to any other attack from your ID until the total attack time is over.
    pub id: Id,
    pub invincible_against: Vec<InvincibleAgainst>,
    pub statuses: Vec<Status>,
    pub show_health_bar: bool,
    pub death_animation: Option<Animation>,
    /// (x, y, delay)
    pub effect: RefCell<Option<(f32, f32, u32)>>,
    pub weapon_action: Option<WeaponAction>,
    pub blocking_direction: Option<Direction>,
    pub animations: Vec<Animation>,
    /// When moving, only the feet should be taken into account, not the head. So this is hitbox
    /// containing width and height based on the bottom of the texture.
    pub move_hitbox: (u32, u32),
}

impl Character {
    pub fn increase_xp(&mut self, xp_to_add: u64, textures: &Textures<'_>, _env: Option<&mut Env>) {
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
        new_x: f32,
        new_y: f32,
        map_data: &[u8],
        dir_to_check: Direction,
    ) -> bool {
        let dimension = self
            .action
            .get_specific_dimension(&self.texture_handler, dir_to_check);
        let new_x = new_x.floor() as i32;
        let new_y = new_y.floor() as i32;
        let new_x = new_x + (dimension.width() as i32 / 2 - self.move_hitbox.0 as i32 / 2);
        let new_y = new_y + (dimension.height() as i32 - self.move_hitbox.1 as i32);
        let initial_x = new_x / MAP_CASE_SIZE;
        let initial_y = new_y / MAP_CASE_SIZE;
        let width = ::std::cmp::max(1, self.move_hitbox.0 as i32 / MAP_CASE_SIZE);
        let height = ::std::cmp::max(1, self.move_hitbox.1 as i32 / MAP_CASE_SIZE);

        match dir_to_check {
            Direction::Down => {
                let y = (height as i32 + initial_y) * MAP_SIZE as i32;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Up => {
                let y = initial_y * MAP_SIZE as i32;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Right => {
                for iy in 0..height {
                    let map_pos =
                        (initial_y + iy) * MAP_SIZE as i32 + initial_x + width + 1;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Left => {
                for iy in 0..height {
                    let map_pos = (initial_y + iy) * MAP_SIZE as i32 + initial_x - 1;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn check_character_move(&self, x: f32, y: f32, character: &Character) -> bool {
        if character.id == self.id {
            return true;
        }
        let self_width = self.move_hitbox.0;
        let self_height = self.move_hitbox.1;
        let self_x = x + (self.width() / 2 - self_width / 2) as f32;
        let self_y = y + (self.height() - self_height) as f32;
        let other_width = character.move_hitbox.0;
        let other_height = character.move_hitbox.1;
        let other_x = character.x() + (character.width() / 2 - other_width / 2) as f32;
        let other_y = character.y() + (character.height() - other_height) as f32;

        !(self_x + self_width as f32 >= other_x
            && self_x <= other_x + other_width as f32
            && self_y + self_height as f32 >= other_y
            && self_y <= other_y + other_height as f32)
    }

    pub fn check_map_pos(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
        new_x: f32,
        new_y: f32,
        ignore_id: Option<Id>,
    ) -> Obstacle {
        let initial_x = ((new_x - map.x).floor() as i32 + self.width() as i32 / 2 - self.move_hitbox.0 as i32 / 2)
            / MAP_CASE_SIZE as i32;
        let initial_y =
            ((new_y - map.y).floor() as i32 + self.height() as i32 - self.move_hitbox.1 as i32) / MAP_CASE_SIZE as i32;
        let width = self.move_hitbox.0 / MAP_CASE_SIZE as u32;
        let height = self.move_hitbox.1 / MAP_CASE_SIZE as u32;

        for y in 0..height {
            let y = (y as i32 + initial_y) * MAP_SIZE as i32;
            for x in 0..width {
                let map_pos = y + initial_x + x as i32;
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

    // FIXME: To compute correctly move in two directions:

    /// `x_add` and `y_add` are used in case you want to move in two directions at once, so when
    /// checking the second direction, you actually already "moved" and don't check a bad position.
    pub fn check_move(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
        // In case of a move on two axes, we have to provide the result of the first move too!
        x_add: f32,
        y_add: f32,
    ) -> (f32, f32) {
        let info = self
            .action
            .get_specific_dimension(&self.texture_handler, *self.action.direction);
        // let (x_add, y_add) = match direction {
        //     Direction::Down
        //         if info.height() as f32 + y + 1. < map.y + MAP_SIZE_WITH_CASE as f32 =>
        //     {
        //         (0., 1.)
        //     }
        //     Direction::Up if y - 1. >= map.y => (0., -1.),
        //     Direction::Left if x - 1. >= map.x => (-1., 0.),
        //     Direction::Right
        //         if info.width() as f32 + x + 1. < map.x + MAP_SIZE_WITH_CASE as f32 =>
        //     {
        //         (1., 0.)
        //     }
        //     _ => return (0., 0.),
        // };
        let x = self.x() + x_add;
        let y = self.y() + y_add;

        fn call(
            self_id: Id,
            c: &Character,
            move_hitbox: &(u32, u32),
            x: f32,
            y: f32,
            width: u32,
            height: u32,
        ) -> bool {
            if self_id == c.id {
                return true;
            }
            let self_x = x + (width / 2 - move_hitbox.0 / 2) as f32;
            let self_y = y + (height - move_hitbox.1) as f32;
            let other_width = c.move_hitbox.0;
            let other_height = c.move_hitbox.1;
            let other_x = c.x() + (c.width() / 2 - other_width / 2) as f32;
            let other_y = c.y() + (c.height() - other_height) as f32;

            let width = move_hitbox.0 as f32;
            let height = move_hitbox.1 as f32;

            !(self_x + width >= other_x
                        && self_x <= other_x + other_width as f32
                        && self_y + height >= other_y
                        && self_y + height <= other_y + other_height as f32)
        }

        let self_x = x + x_add;
        let self_y = y + y_add;
        // NPC moves are a bit more restricted than players'.
        let can_move = if self.kind.is_player() {
            let width = self.width();
            let height = self.height();
            self.check_hitbox(self_x - map.x, self_y - map.y, &map.data, *self.action.direction)
                && npcs.iter().all(|n| {
                    call(
                        self.id,
                        n.character(),
                        &self.move_hitbox,
                        self_x,
                        self_y,
                        width,
                        height,
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
                    )
                })
        } else {
            self.check_hitbox(self_x - map.x, self_y - map.y, &map.data, *self.action.direction)
                && npcs
                    .iter()
                    .all(|n| self.check_character_move(self_x, self_y, n.character()))
                && players
                    .iter()
                    .all(|n| self.check_character_move(self_x, self_y, &n))
        };
        if can_move {
            (x_add, y_add)
        } else {
            (0., 0.)
        }
    }

    /// `x_add` and `y_add` are used in case you want to move in two directions at once, so when
    /// checking the second direction, you actually already "moved" and don't check a bad position.
    pub fn inner_check_move(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
        primary_direction: DirectionAndStrength,
        secondary_direction: Option<DirectionAndStrength>,
        x_add: f32,
        y_add: f32,
    ) -> (f32, f32) {
        let (x1, y1) = primary_direction.convert_to_x_y();
        let angle = if let Some(secondary_direction) = secondary_direction {
            let (x2, y2) = secondary_direction.convert_to_x_y();
            (x2 + x1).atan2(y2 + y1)
        } else {
            x1.atan2(y1)
        };
        let x = angle.sin();
        let y = angle.cos();
        self.check_move(map, players, npcs, x_add + x, y_add + y)
    }

    /// `x_add` and `y_add` are used in case you want to move in two directions at once, so when
    /// checking the second direction, you actually already "moved" and don't check a bad position.
    pub fn inner_apply_move(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
        x_add: f32,
        y_add: f32,
    ) -> (f32, f32) {
        if self.action.movement.is_none() {
            (0., 0.)
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
        let (tile_x, tile_y, tile_width, tile_height, draw_width, draw_height) =
            self.action.compute_current(&self.texture_handler);
        if self.is_dead() {
            if let Some(ref death) = self.death_animation {
                death.draw(
                    system,
                    self.x + (draw_width / 2) as f32,
                    self.y + (draw_height / 2) as f32,
                );
                return;
            }
        }
        let mut x = self.x - system.x();
        let mut y = self.y - system.y();
        let is_in_viewport = x + draw_width as f32 >= 0.
            && x < system.width() as f32
            && y + draw_height as f32 >= 0.
            && y < system.height() as f32;
        if !is_in_viewport {
            return;
        }
        if let Some(direction) = self.blocking_direction {
            self.weapon.draw_blocking(system, direction);
        } else if let Some(ref action) = self.weapon_action {
            if let Some((x_add, y_add)) = action.get_attack_by_move_target() {
                x += x_add;
                y += y_add;
            } else {
                self.weapon.draw(system, action);
            }
        }

        system.copy_to_canvas(
            self.texture_handler.texture,
            Rect::new(tile_x, tile_y, tile_width, tile_height),
            Rect::new(x as i32, y as i32, draw_width, draw_height),
        );

        for animation in self.animations.iter() {
            animation.draw(
                system,
                self.x + (draw_width / 2) as f32,
                self.y + (draw_height - animation.sprite_display_height / 2) as f32,
            );
        }
        if debug {
            system
                .canvas
                .draw_rect(Rect::new(x as _, y as _, draw_width, draw_height))
                .unwrap();
            system
                .canvas
                .draw_rect(Rect::new(
                    (x + (self.width() / 2 - self.move_hitbox.0 / 2) as f32) as i32,
                    (y + (self.height() - self.move_hitbox.1) as f32) as i32,
                    self.move_hitbox.0,
                    self.move_hitbox.1,
                ))
                .unwrap();
        }
        // if let Some(matrix) = weapon.compute_angle() {
        //     for (x, y) in matrix.iter() {
        //         canvas.fill_rect(Rect::new(x - screen.x, y - screen.y, 8, 8));
        //     }
        // }

        if self.show_health_bar && !self.stats.health.is_full() {
            system.health_bar.draw(
                self.x + ((draw_width as i32 - system.health_bar.width as i32) / 2) as f32,
                self.y - (system.health_bar.height + 2) as f32,
                self.stats.health.pourcent(),
                system,
            );
        }

        let x = self.x + (self.width() / 2) as f32;
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
        if remaining_stamina >= self.weapon.weight() as _ {
            self.weapon_action = self.weapon.use_it(*self.action.direction);
            self.blocking_direction = None;

            self.set_weapon_pos();
            self.stats.stamina.subtract(self.weapon.weight() as _);
        }
    }

    pub fn is_attacking(&self) -> bool {
        self.weapon_action.is_some()
    }

    pub fn stop_attack(&mut self) {
        self.weapon_action = None;
    }

    pub fn apply_move(
        &self,
        map: &Map,
        elapsed: u32,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
    ) -> (f32, f32) {
        if self.is_dead() {
            return (0., 0.);
        }
        let mut tmp = self.move_delay + elapsed;
        let mut stamina = self.stats.stamina.clone();
        let mut x = 0.;
        let mut y = 0.;

        if let Some(ref mut effect) = &mut *self.effect.borrow_mut() {
            while tmp > effect.2 && (effect.0 != 0. || effect.1 != 0.) {
                if effect.0 != 0. {
                    let (x1, _) = self.check_move(
                        map,
                        players,
                        npcs,
                        x,
                        y,
                    );
                    if x1 != 0. {
                        x += x1;
                        effect.0 += x1 * -1.;
                    } else {
                        effect.0 = 0.;
                        effect.1 = 0.;
                        break;
                    }
                }
                if effect.1 != 0. {
                    let (_, y1) = self.check_move(
                        map,
                        players,
                        npcs,
                        x,
                        y,
                    );
                    if y1 != 0. {
                        y += y1;
                        effect.1 += y1 * -1.;
                    } else {
                        effect.0 = 0.;
                        effect.1 = 0.;
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
                if x1 != 0. || y1 != 0. {
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

    pub fn update(&mut self, elapsed: u32, x: f32, y: f32, _env: Option<&mut Env>) {
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

        let _env_stamina = self.stats.stamina.refresh(elapsed);
        let _env_health = self.stats.health.refresh(elapsed);
        let _env_mana = self.stats.mana.refresh(elapsed);
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
            if effect.0 != 0. || effect.1 != 0. {
                *self.effect.borrow_mut() = Some(effect);
            }
        } else {
            while self.move_delay > self.speed {
                let stamina_value = self.stats.stamina.value();
                if self.is_running && stamina_value > 0 {
                    self.stats.stamina.subtract(1);
                    self.is_running = stamina_value - 1 > 0;
                }
                self.move_delay -= self.speed;
            }

            // Normally, when we block we set the weapon_action to None.
            // if self.blocking_direction.is_none() {
            if let Some(mut weapon_action) = self.weapon_action.take() {
                weapon_action.duration += elapsed;
                if weapon_action.duration <= weapon_action.total_duration {
                    self.weapon_action = Some(weapon_action);
                }
            }
            // }
        }

        if let Some(ref mut pos) = self.action.movement {
            self.tile_delay += elapsed;
            let tile_duration = if self.is_running {
                self.tile_duration / 2
            } else {
                self.tile_duration
            };
            while self.tile_delay > tile_duration {
                // We now update the animation!
                *pos += 1;
                self.tile_delay -= tile_duration;
            }
            let nb_animations =
                self.texture_handler.actions_moving[*self.action.direction as usize].1 as u32;
            if *pos > nb_animations {
                *pos %= nb_animations;
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
            if self.invincible_against[i].remaining_time <= elapsed as u32 {
                self.invincible_against.remove(i);
            } else {
                self.invincible_against[i].remaining_time -= elapsed as u32;
                i += 1;
            }
        }
    }

    fn set_weapon_pos(&mut self) {
        if self.is_blocking() {
            // To set the direction of the blocking.
            self.block();

            let (_, _, _, _, draw_width, draw_height) =
                self.action.compute_current(&self.texture_handler);
            let draw_width = draw_width;
            let draw_height = draw_height;
            let width = self.weapon.width();
            let height = self.weapon.height();
            let (x, y) = match *self.action.direction {
                Direction::Up => (self.x + (width + 2) as f32, self.y - (height + 3) as f32),
                Direction::Down => (
                    self.x + (width / 2) as f32,
                    self.y + (draw_height - 2) as f32,
                ),
                Direction::Left => (self.x - (width - 4) as f32, self.y),
                Direction::Right => (self.x + draw_width as f32, self.y),
            };
            self.weapon.set_pos(x, y);
        } else {
            let (_, _, _, _, draw_width, draw_height) =
                self.action.compute_current(&self.texture_handler);
            let draw_width = draw_width as i32;
            let draw_height = draw_height as i32;
            let width = self.weapon.width() as i32;
            let height = self.weapon.height() as i32;
            let (x, y) = match *self.action.direction {
                Direction::Up => (self.x + (draw_width / 2 - 3) as f32, self.y - height as f32),
                Direction::Down => (
                    self.x + (draw_width / 2 - 4) as f32,
                    self.y + (draw_height - height) as f32,
                ),
                Direction::Left => (self.x - 2., self.y + (draw_height / 2 - height + 2) as f32),
                Direction::Right => (
                    self.x + (draw_width - width + 2) as f32,
                    self.y + (draw_height / 2 - height) as f32,
                ),
            };
            self.weapon.set_pos(x, y);
        }
    }

    fn check_attack_by_move_intersection(&self, attacker: &Character) -> bool {
        let x_overlap = if self.x > attacker.x {
            attacker.x - self.x < attacker.width() as _
        } else {
            self.x - attacker.x < self.width() as _
        };
        if !x_overlap {
            return false;
        }
        // For y, we need to "switch" where we add the height because the y axis is reversed.
        if self.y > attacker.y {
            self.y - attacker.y < self.height() as _
        } else {
            attacker.y - self.y < attacker.height() as _
        }
    }

    pub fn check_intersection(
        &self,
        attacker: &Character,
        matrix: &mut Option<ConvexPolygon>,
        textures: &Textures<'_>,
    ) -> i32 {
        if self.is_dead()
            || attacker.id == self.id
            || self.invincible_against.iter().any(|e| e.id == attacker.id)
        {
            return 0;
        }
        let (_tile_x, _tile_y, _, _, width, height) =
            self.action.compute_current(&self.texture_handler);
        let w_biggest = ::std::cmp::max(attacker.weapon.height(), attacker.weapon.width()) as f32;
        let attacker_direction = attacker.get_direction();
        let weapon_x = if attacker_direction == Direction::Right {
            attacker.weapon.x + w_biggest
        } else {
            attacker.weapon.x
        };
        let weapon_y = if attacker_direction == Direction::Down {
            attacker.weapon.y + w_biggest
        } else {
            attacker.weapon.y
        };

        if weapon_x + w_biggest < self.x
            || weapon_x - w_biggest > self.x + width as f32
            || weapon_y + w_biggest < self.y
            || weapon_y - w_biggest > self.y + height as f32
        {
            // The weapon is too far from this character, no need to check further!
            return 0;
        }

        let has_intersection = if attacker
            .weapon_action
            .as_ref()
            .map(|a| a.is_attack_by_move())
            .unwrap_or(false)
        {
            self.check_attack_by_move_intersection(attacker)
        } else {
            if matrix.is_none() {
                *matrix = attacker.weapon.compute_angle(&attacker.weapon_action);
            }
            if let Some(ref matrix) = matrix {
                self.texture_handler.check_intersection(
                    textures,
                    matrix,
                    *self.action.direction,
                    self.action.movement.is_some(),
                    (self.x, self.y),
                )
            } else {
                false
            }
        };
        if !has_intersection {
            return 0;
        }
        // TODO: add element effects on attacks.
        // TODO2: if you attack with fire effect on a fire monster, it heals it!
        let attack = if attacker.weapon.attack >= 0 {
            let attack = attacker.weapon.attack as u32 + attacker.stats.attack;
            let attack = if self.is_blocking() {
                let dir = self.get_direction();
                {
                    let mut effect = self.effect.borrow_mut();
                    if effect.is_none() {
                        // We want the character to be moved by 6 cases.
                        let distance = MAP_CASE_SIZE * 6;
                        // We want the "animation" to last for half a second.
                        let dur = ONE_SECOND / 2 / distance as u32;
                        *effect = Some(match dir {
                            Direction::Up => (0., distance as f32, dur),
                            Direction::Down => (0., (distance * -1) as f32, dur),
                            Direction::Right => ((distance * -1) as f32, 0., dur),
                            Direction::Left => (distance as f32, 0., dur),
                        });
                    }
                }
                if dir.is_opposite(attacker_direction) {
                    // They're facing each other, full block on the attack!
                    attack / 2
                } else if dir.is_adjacent(attacker_direction) {
                    // Partially blocked, only 25% of the attack is removed
                    attack * 3 / 4
                } else {
                    // The attack is on the back, full damage!
                    attack
                }
            } else {
                attack
            };
            let attack = if attack == 0 { 1 } else { attack };
            attack as i32
        } else {
            // Since it's supposed to heal, we don't take into account anything except
            // the weapon "attack".
            attacker.weapon.attack
        };
        return attack;
    }

    pub fn update_attack_info(&mut self, attacker_id: Id, attacker_weapon_time: u32, attack: i32) {
        if attack > 0 {
            self.stats.health.subtract(attack as _);
        } else if attack < 0 {
            self.stats.health.add((attack * -1) as _);
        }
        // TODO: not the same display if attack is negative (meaning you gain back health!).
        if !self.stats.health.is_empty() {
            self.invincible_against
                .push(InvincibleAgainst::new(attacker_id, attacker_weapon_time));
            // TODO: add defense on characters and make computation here (also add dodge
            // computation and the other stuff...)
            self.statuses
                .push(Status::new(attack.to_string(), Color::RGB(255, 0, 0)));
        }
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
        self.blocking_direction.is_some()
    }
    pub fn block(&mut self) {
        let dir = self.get_direction();
        self.weapon_action = None;
        if self.weapon.can_block() {
            self.blocking_direction = Some(dir);
            self.set_weapon_pos();
        }
    }
    pub fn stop_block(&mut self) {
        self.blocking_direction = None;
    }

    pub fn get_direction(&self) -> Direction {
        *self.action.direction
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
        // When you get resurrected "by yourself", you lose 30% of your xp.
        let third = self.xp_to_next_level / 30;
        if third < self.xp {
            self.xp -= third;
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
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}
