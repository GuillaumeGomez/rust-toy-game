use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::enemy::Enemy;
use crate::map::Map;
use crate::player::Player;
use crate::status::Status;
use crate::system::System;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::weapon::Weapon;
use crate::{GetDimension, Id, MAP_CASE_SIZE, MAP_SIZE};

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
    ) -> (i32, i32, i32, i32) {
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
                info.width() as i32,
                info.height() as i32,
            )
        } else {
            let info = &textures.actions_standing[self.direction as usize];
            (info.x, info.y, info.width() as i32, info.height() as i32)
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

pub struct Character<'a> {
    pub action: Action,
    pub x: i32,
    pub y: i32,
    pub total_health: u32,
    pub health: u32,
    pub total_mana: u32,
    pub mana: u32,
    pub total_stamina: u32,
    pub stamina: u32,
    pub xp_to_next_level: u32,
    pub xp: u32,
    pub texture_handler: TextureHandler<'a>,
    pub weapon: Option<Weapon<'a>>,
    pub is_running: bool,
    /// This ID is used when this character is attacking someone else. This "someone else" will
    /// invincible to any other attack from your ID until the total attack time is over.
    pub id: Id,
    pub invincible_against: HashMap<Id, i32>,
    pub statuses: Vec<Status<'a>>,
}

impl<'a> Character<'a> {
    fn check_hitbox(
        &self,
        new_x: i32,
        new_y: i32,
        map_data: &[u8],
        dir_to_check: Direction,
    ) -> bool {
        let initial_y = new_y / MAP_CASE_SIZE;
        let initial_x = new_x / MAP_CASE_SIZE;
        let dimension = self.action.get_dimension(&self.texture_handler);
        let height = dimension.height() as i32 / MAP_CASE_SIZE;
        let width = dimension.width() as i32 / MAP_CASE_SIZE;

        match dir_to_check {
            Direction::Down => {
                let y = (height + initial_y) * MAP_SIZE as i32;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
                let y = (height + initial_y + 1) * MAP_SIZE as i32;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Up => {
                let y = (initial_y + 1) * MAP_SIZE as i32;
                for ix in 0..width {
                    let map_pos = y + initial_x + ix;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Right => {
                for iy in 1..height {
                    let map_pos = (initial_y + iy) * MAP_SIZE as i32 + initial_x + width;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
            Direction::Left => {
                for iy in 1..height {
                    let map_pos = (initial_y + iy) * MAP_SIZE as i32 + initial_x;
                    if map_pos < 0 || map_data.get(map_pos as usize).unwrap_or(&1) != &0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn check_character_move(&self, x: i32, y: i32, character: &Character) -> bool {
        character.id == self.id
            || !(self.width() as i32 + x >= character.x
                && x <= character.x + character.width() as i32
                && self.height() as i32 + y >= character.y
                && y <= character.y + character.height() as i32)
    }

    fn check_move(
        &self,
        direction: Direction,
        map: &Map,
        players: &[Player],
        npcs: &[Enemy],
        x_add: i32,
        y_add: i32,
    ) -> (i32, i32) {
        let (info, _) = &self.texture_handler.actions_moving[direction as usize];
        let self_x = self.x + x_add;
        let self_y = self.y + y_add;
        let (x_add, y_add) = match direction {
            Direction::Down
                if self_y + info.height() as i32 + 1 < map.y + MAP_SIZE as i32 * MAP_CASE_SIZE =>
            {
                (0, 1)
            }
            Direction::Up if self_y - 1 >= map.y => (0, -1),
            Direction::Left if self_x - 1 >= map.x => (-1, 0),
            Direction::Right
                if self_x + info.width() as i32 + 1 < map.x + MAP_SIZE as i32 * MAP_CASE_SIZE =>
            {
                (1, 0)
            }
            _ => return (0, 0),
        };
        let x = self_x + x_add;
        let y = self_y + y_add;
        if self.check_hitbox(x - map.x, y - map.y, &map.data, Direction::Down)
            && npcs.iter().all(|n| self.check_character_move(x, y, &n))
            && players.iter().all(|p| self.check_character_move(x, y, &p))
        {
            (x_add, y_add)
        } else {
            (0, 0)
        }
    }

    pub fn inner_apply_move(&self, map: &Map, players: &[Player], npcs: &[Enemy]) -> (i32, i32) {
        if self.action.movement.is_none() {
            return (0, 0);
        }
        let (mut x, mut y) = self.check_move(self.action.direction, map, players, npcs, 0, 0);
        if let Some(second) = self.action.secondary {
            let (x2, y2) = self.check_move(second, map, players, npcs, x, y);
            x += x2;
            y += y2;
        }
        (x, y)
    }

    pub fn draw(&mut self, system: &mut System) {
        let (tile_x, tile_y, tile_width, tile_height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        let x = self.x - system.x();
        let y = self.y - system.y();
        if x + tile_width >= 0
            && x < system.width() as i32
            && y + tile_height >= 0
            && y < system.height() as i32
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
        if let Some(ref mut weapon) = self.weapon {
            // if let Some(matrix) = weapon.compute_angle() {
            //     for (x, y) in matrix.iter() {
            //         canvas.fill_rect(Rect::new(x - screen.x, y - screen.y, 8, 8));
            //     }
            // }
            weapon.draw(system);
        }

        let x = self.x + self.width() as i32 / 2;
        let mut it = 0;

        while it < self.statuses.len() {
            self.statuses[it].draw(system, x, self.y);
            if self.statuses[it].should_be_removed() {
                self.statuses.remove(it);
                continue;
            }
            it += 1;
        }

        // We now update the animation!
        if let Some(ref mut pos) = self.action.movement {
            *pos += 1;
        } else {
            if self.stamina < self.total_stamina {
                self.stamina += 1;
            }
            return;
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

    pub fn apply_move(&self, map: &Map, players: &[Player], npcs: &[Enemy]) -> (i32, i32) {
        let (mut x, mut y) = self.inner_apply_move(map, players, npcs);
        if (x != 0 || y != 0) && self.is_running {
            let (x2, y2) = self.inner_apply_move(map, players, npcs);
            x += x2;
            y += y2;
        }
        (x, y)
    }

    pub fn update(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
        if x != 0 || y != 0 {
            self.set_weapon_pos();
        }
        if self.is_running {
            if self.stamina > 0 {
                self.stamina -= 1;
                if self.stamina == 0 {
                    self.is_running = false;
                }
            }
        } else if self.stamina < self.total_stamina {
            self.stamina += 1;
        }
        if self.invincible_against.is_empty() {
            return;
        }
        let mut to_remove = Vec::new();
        for (key, value) in self.invincible_against.iter_mut() {
            *value -= 1;
            if *value <= 0 {
                to_remove.push(*key);
            }
        }
        for key in to_remove {
            self.invincible_against.remove(&key);
        }
    }

    fn set_weapon_pos(&mut self) {
        if let Some(ref mut weapon) = self.weapon {
            let (tile_x, tile_y, tile_width, tile_height) = self
                .action
                .compute_current(self.is_running, &self.texture_handler);
            let width = weapon.width() as i32;
            let height = weapon.height() as i32;
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

    // TODO: instead of this, pass a "impl Iterator<Character>" argument and go through all of them
    pub fn check_intersection<'b>(
        &mut self,
        character: &Character<'a>,
        font: &'b Font<'b, 'static>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) {
        if self.invincible_against.contains_key(&character.id) {
            return;
        }
        let weapon = return_if_none!(&character.weapon);
        let (tile_x, tile_y, width, height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        let width = width as i32;
        let height = height as i32;

        let w_height = weapon.height() as i32;
        let w_width = weapon.width() as i32;
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
            || weapon.x - w_biggest > self.x + width
            || weapon_y + w_biggest < self.y
            || weapon_y - w_biggest > self.y + height
        {
            // The weapon is too far from this character, no need to check further!
            return;
        }

        let matrix = return_if_none!(weapon.compute_angle());
        if self.texture_handler.check_intersection(
            &matrix,
            (tile_x, tile_y),
            (width, height),
            (self.x, self.y),
        ) {
            self.invincible_against
                .insert(character.id, weapon.total_time);
            // TODO: add defense on characters and make computation here (also add dodge computation
            // and the other stuff...)
            self.statuses.push(Status::new(
                font,
                texture_creator,
                10,
                &weapon.attack.to_string(),
                Color::RGB(255, 0, 0),
            ));
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
