use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::map::Map;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::weapon::Weapon;
use crate::{GetDimension, MAP_CASE_SIZE, MAP_SIZE};

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
    // /// This ID is used when this character is attacking someone else. This "someone else" will
    // /// invincible to any other attack from your ID until the total attack time is over.
    // pub id: usize,
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

    fn check_move(&mut self, direction: Direction, map: &Map) -> bool {
        let (info, _) = &self.texture_handler.actions_moving[direction as usize];
        match direction {
            Direction::Down => {
                if self.y + info.height() as i32 + 1 < map.y + MAP_SIZE as i32 * MAP_CASE_SIZE {
                    if self.check_hitbox(
                        self.x - map.x,
                        self.y + 1 - map.y,
                        &map.data,
                        Direction::Down,
                    ) {
                        self.y += 1;
                        return true;
                    }
                }
            }
            Direction::Up => {
                if self.y - 1 >= map.y {
                    if self.check_hitbox(
                        self.x - map.x,
                        self.y - 1 - map.y,
                        &map.data,
                        Direction::Up,
                    ) {
                        self.y -= 1;
                        return true;
                    }
                }
            }
            Direction::Left => {
                if self.x - 1 >= map.x {
                    if self.check_hitbox(
                        self.x - 1 - map.x,
                        self.y - map.y,
                        &map.data,
                        Direction::Left,
                    ) {
                        self.x -= 1;
                        return true;
                    }
                }
            }
            Direction::Right => {
                if self.x + info.width() as i32 + 1 < map.x + MAP_SIZE as i32 * MAP_CASE_SIZE {
                    if self.check_hitbox(
                        self.x + 1 - map.x,
                        self.y - map.y,
                        &map.data,
                        Direction::Right,
                    ) {
                        self.x += 1;
                        return true;
                    }
                }
            }
        }
        false
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

    pub fn inner_apply_move(&mut self, map: &Map) -> bool {
        if self.action.movement.is_none() {
            return false;
        }
        let moved = self.check_move(self.action.direction, map);
        let moved = if let Some(second) = self.action.secondary {
            self.check_move(second, map) || moved
        } else {
            moved
        };
        if moved {
            self.set_weapon_pos();
        }
        moved
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        let (tile_x, tile_y, tile_width, tile_height) = self
            .action
            .compute_current(self.is_running, &self.texture_handler);
        let debug = self.xp == 150;
        if (self.x + tile_width < screen.x || self.x > screen.x + screen.width() as i32)
            && (self.y + tile_height < screen.y || self.y > screen.y + screen.height() as i32)
        {
            // No need to draw if we don't see the character.
            return;
        }
        canvas
            .copy(
                &self.texture_handler.texture,
                Rect::new(tile_x, tile_y, tile_width as u32, tile_height as u32),
                Rect::new(
                    self.x - screen.x,
                    self.y - screen.y,
                    tile_width as u32,
                    tile_height as u32,
                ),
            )
            .expect("copy character failed");
        if let Some(ref mut weapon) = self.weapon {
            // if let Some(matrix) = weapon.compute_angle() {
            //     for (x, y) in matrix.iter() {
            //         canvas.fill_rect(Rect::new(x - screen.x, y - screen.y, 8, 8));
            //     }
            // }
            weapon.draw(canvas, screen);
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

    pub fn apply_move(&mut self, map: &Map) {
        if self.inner_apply_move(map) {
            if self.is_running {
                self.inner_apply_move(map);
                if self.stamina > 0 {
                    self.stamina -= 1;
                    if self.stamina == 0 {
                        self.is_running = false;
                    }
                }
                return;
            }
        }
        if self.stamina < self.total_stamina {
            self.stamina += 1;
        }
    }

    pub fn check_intersection(&mut self, weapon: &Weapon<'a>) {
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
            println!("intersection!");
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
