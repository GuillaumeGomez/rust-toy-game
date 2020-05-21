use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::death_animation::DeathAnimation;
use crate::enemy::Enemy;
use crate::map::Map;
use crate::player::Player;
use crate::reward::RewardInfo;
use crate::stat::Stat;
use crate::status::Status;
use crate::system::System;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::weapon::Weapon;
use crate::{GetDimension, GetPos, Id, MAP_CASE_SIZE, MAP_SIZE};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharacterKind {
    Player,
    Enemy, // TODO: Enemy is just temporary, it'll be replaced by an id for each kind of monsters
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[repr(usize)]
pub enum Direction {
    Down = 0,
    Up = 1,
    Left = 2,
    Right = 3,
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct Action {
    pub direction: Direction,
    pub secondary: Option<Direction>,
    pub movement: Option<u64>,
}

impl Action {
    /// Returns `(x, y, width, height)`.
    pub fn compute_current(
        &self,
        is_running: bool,
        textures: &TextureHandler<'_>,
    ) -> (i32, i32, i64, i64) {
        if let Some(ref pos) = self.movement {
            let (info, nb_animations) = &textures.actions_moving[self.direction as usize];
            let pos = if is_running {
                (pos % 30) as i32 / (30 / nb_animations)
            } else {
                (pos % 60) as i32 / (60 / nb_animations)
            };
            (
                pos * info.incr_to_next + info.x,
                info.y,
                info.width() as i64,
                info.height() as i64,
            )
        } else {
            let info = &textures.actions_standing[self.direction as usize];
            (info.x, info.y, info.width() as i64, info.height() as i64)
        }
    }

    pub fn get_dimension<'a>(&self, textures: &'a TextureHandler<'_>) -> &'a Dimension {
        if let Some(_) = self.movement {
            &textures.actions_moving[self.direction as usize].0
        } else {
            &textures.actions_standing[self.direction as usize]
        }
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

pub struct Character<'a> {
    pub action: Action,
    pub x: i64,
    pub y: i64,
    pub kind: CharacterKind,
    pub health: Stat,
    pub mana: Stat,
    pub stamina: Stat,
    pub xp_to_next_level: u64,
    pub xp: u64,
    // TODO: Instead of creating a new TextureHandler every time, might be better to use references!
    pub texture_handler: TextureHandler<'a>,
    pub weapon: Option<Weapon<'a>>,
    pub is_running: bool,
    /// How much time you need to move of 1.
    pub speed: u64,
    /// When "delay" is superior than "speed", we trigger the movement.
    pub move_delay: u64,
    /// This ID is used when this character is attacking someone else. This "someone else" will
    /// invincible to any other attack from your ID until the total attack time is over.
    pub id: Id,
    pub invincible_against: Vec<InvincibleAgainst>,
    pub statuses: Vec<Status<'a>>,
    pub show_health_bar: bool,
    pub death_animation: Option<DeathAnimation<'a>>,
}

impl<'a> Character<'a> {
    fn check_hitbox(
        &self,
        new_x: i64,
        new_y: i64,
        map_data: &[u8],
        dir_to_check: Direction,
    ) -> bool {
        let initial_y = new_y / MAP_CASE_SIZE;
        let initial_x = new_x / MAP_CASE_SIZE;
        let dimension = self.action.get_dimension(&self.texture_handler);
        let height = dimension.height() as i64 / MAP_CASE_SIZE;
        let width = dimension.width() as i64 / MAP_CASE_SIZE;

        match dir_to_check {
            Direction::Down => {
                let y = (height + initial_y) * MAP_SIZE as i64;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Up => {
                let y = (initial_y + 1) * MAP_SIZE as i64;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Right => {
                for iy in 1..height {
                    let map_pos = (initial_y + iy) * MAP_SIZE as i64 + initial_x + width;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Left => {
                for iy in 1..height {
                    let map_pos = (initial_y + iy) * MAP_SIZE as i64 + initial_x;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn check_character_move(&self, x: i64, y: i64, character: &Character) -> bool {
        character.id == self.id
            || !(self.width() as i64 + x >= character.x
                && x <= character.x + character.width() as i64
                && self.height() as i64 + y >= character.y
                && y <= character.y + character.height() as i64)
    }

    pub fn check_pos(
        &self,
        direction: Direction,
        map: &Map,
        players: &[Player],
        npcs: &[Enemy],
        x: i64,
        y: i64,
    ) -> bool {
        self.check_hitbox(x - map.x, y - map.y, &map.data, direction)
            && npcs.iter().all(|n| self.check_character_move(x, y, &n))
            && players.iter().all(|p| self.check_character_move(x, y, &p))
    }

    fn check_move(
        &self,
        direction: Direction,
        map: &Map,
        players: &[Player],
        npcs: &[Enemy],
        x_add: i64,
        y_add: i64,
    ) -> (i64, i64) {
        let (info, _) = &self.texture_handler.actions_moving[direction as usize];
        let self_x = self.x + x_add;
        let self_y = self.y + y_add;
        let (x_add, y_add) = match direction {
            Direction::Down
                if self_y + info.height() as i64 + 1 < map.y + MAP_SIZE as i64 * MAP_CASE_SIZE =>
            {
                (0, 1)
            }
            Direction::Up if self_y - 1 >= map.y => (0, -1),
            Direction::Left if self_x - 1 >= map.x => (-1, 0),
            Direction::Right
                if self_x + info.width() as i64 + 1 < map.x + MAP_SIZE as i64 * MAP_CASE_SIZE =>
            {
                (1, 0)
            }
            _ => return (0, 0),
        };
        if self.check_pos(
            direction,
            map,
            players,
            npcs,
            self_x + x_add,
            self_y + y_add,
        ) {
            (x_add, y_add)
        } else {
            (0, 0)
        }
    }

    pub fn inner_apply_move(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Enemy],
        x_add: i64,
        y_add: i64,
    ) -> (i64, i64) {
        if self.action.movement.is_none() {
            return (0, 0);
        }
        let (mut x, mut y) =
            self.check_move(self.action.direction, map, players, npcs, x_add, y_add);
        if let Some(second) = self.action.secondary {
            let (x2, y2) = self.check_move(second, map, players, npcs, x + x_add, y + y_add);
            x += x2;
            y += y2;
        }
        (x, y)
    }

    pub fn draw(&mut self, system: &mut System) {
        let (tile_x, tile_y, tile_width, tile_height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        if self.is_dead() {
            if let Some(ref death) = self.death_animation {
                death.draw(system, self.x + tile_width / 2, self.y + tile_height / 2);
                return;
            }
        }
        let x = (self.x - system.x()) as i32;
        let y = (self.y - system.y()) as i32;
        if x + tile_width as i32 >= 0
            && x < system.width()
            && y + tile_height as i32 >= 0
            && y < system.height()
        {
            system
                .canvas
                .copy(
                    &self.texture_handler.texture,
                    Rect::new(tile_x, tile_y, tile_width as u32, tile_height as u32),
                    Rect::new(x, y, tile_width as u32, tile_height as u32),
                )
                .expect("copy character failed");
        }
        if let Some(ref weapon) = self.weapon {
            // if let Some(matrix) = weapon.compute_angle() {
            //     for (x, y) in matrix.iter() {
            //         canvas.fill_rect(Rect::new(x - screen.x, y - screen.y, 8, 8));
            //     }
            // }
            weapon.draw(system);
        }

        if self.show_health_bar && !self.health.is_full() {
            system.health_bar.draw(
                self.x + (tile_width as i32 - system.health_bar.width as i32) as i64 / 2,
                self.y - (system.health_bar.height + 2) as i64,
                self.health.pourcent(),
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

    // TODO: add stamina consumption when attacking
    pub fn attack(&mut self) {
        if let Some(ref mut weapon) = self.weapon {
            weapon.use_it(self.action.direction);
            self.set_weapon_pos();
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
        npcs: &[Enemy],
    ) -> (i64, i64) {
        if self.is_dead() {
            return (0, 0);
        }
        let mut tmp = self.move_delay + elapsed;
        let mut stamina = self.stamina.clone();
        let mut x = 0;
        let mut y = 0;

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
        (x, y)
    }

    pub fn update(&mut self, elapsed: u64, x: i64, y: i64) {
        if self.is_dead() {
            if let Some(ref mut death) = self.death_animation {
                death.update(elapsed);
            }
            return;
        }
        self.x += x;
        self.y += y;
        if x != 0 || y != 0 {
            self.set_weapon_pos();
        }

        self.stamina.refresh(elapsed);
        self.health.refresh(elapsed);
        self.mana.refresh(elapsed);
        self.move_delay += elapsed;
        while self.move_delay > self.speed {
            // We now update the animation!
            if let Some(ref mut pos) = self.action.movement {
                *pos += 1;
            }
            let stamina_value = self.stamina.value();
            if self.is_running && stamina_value > 0 {
                self.stamina.subtract(1);
                self.is_running = stamina_value - 1 > 0;
            }
            self.move_delay -= self.speed;
        }

        if let Some(ref mut weapon) = self.weapon {
            weapon.update(elapsed);
        }

        // We update the statuses display
        for status in self.statuses.iter_mut() {
            status.update(elapsed);
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
            let (_, _, tile_width, tile_height) = self
                .action
                .compute_current(self.is_running, &self.texture_handler);
            let width = weapon.width() as i64;
            let height = weapon.height() as i64;
            let (x, y) = match self.action.direction {
                Direction::Up => (self.x + tile_width / 2 - 3, self.y - height),
                Direction::Down => (self.x + tile_width / 2 - 4, self.y + tile_height - height),
                Direction::Left => (self.x - 2, self.y + tile_height / 2 - height + 2),
                Direction::Right => (
                    self.x + tile_width - width + 2,
                    self.y + tile_height / 2 - height,
                ),
            };
            weapon.set_pos(x, y);
        }
    }

    pub fn check_intersection<'b>(
        &mut self,
        character_id: Id,
        weapon: &Weapon<'a>,
        matrix: &mut Option<Vec<(i64, i64)>>,
        font: &'b Font<'b, 'static>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> i32 {
        if self.is_dead()
            || character_id == self.id
            || self.invincible_against.iter().any(|e| e.id == character_id)
        {
            return 0;
        }
        let (tile_x, tile_y, width, height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        let w_height = weapon.height() as i64;
        let w_width = weapon.width() as i64;
        let w_biggest = if w_height > w_width {
            w_height
        } else {
            w_width
        };
        let weapon_y = if self.action.direction == Direction::Down {
            weapon.y + w_biggest
        } else {
            weapon.y
        };

        if weapon.x + w_biggest < self.x
            || weapon.x - w_biggest > self.x + width as i64
            || weapon_y + w_biggest < self.y
            || weapon_y - w_biggest > self.y + height as i64
        {
            // The weapon is too far from this character, no need to check further!
            return 0;
        }

        if matrix.is_none() {
            *matrix = weapon.compute_angle();
        }
        if let Some(ref matrix) = matrix {
            if self.texture_handler.check_intersection(
                &matrix,
                (tile_x, tile_y),
                (width as i32, height as i32),
                (self.x, self.y),
            ) {
                if weapon.attack >= 0 {
                    self.health.subtract(weapon.attack as u64);
                } else {
                    self.health.add((weapon.attack * -1) as u64);
                }
                if !self.health.is_empty() {
                    self.invincible_against
                        .push(InvincibleAgainst::new(character_id, weapon.total_time));
                    // TODO: add defense on characters and make computation here (also add dodge
                    // computation and the other stuff...)
                    self.statuses.push(Status::new(
                        font,
                        texture_creator,
                        &weapon.attack.to_string(),
                        Color::RGB(255, 0, 0),
                    ));
                }
                return weapon.attack;
            }
        }
        0
    }

    pub fn is_dead(&self) -> bool {
        self.health.is_empty()
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
}

impl<'a> GetDimension for Character<'a> {
    fn width(&self) -> u32 {
        if self.action.movement.is_none() {
            self.texture_handler.actions_standing[self.action.direction as usize].width()
        } else {
            self.texture_handler.actions_moving[self.action.direction as usize]
                .0
                .width()
        }
    }

    fn height(&self) -> u32 {
        if self.action.movement.is_none() {
            self.texture_handler.actions_standing[self.action.direction as usize].height()
        } else {
            self.texture_handler.actions_moving[self.action.direction as usize]
                .0
                .height()
        }
    }
}

impl<'a> GetPos for Character<'a> {
    fn x(&self) -> i64 {
        self.x
    }

    fn y(&self) -> i64 {
        self.y
    }
}
