use std::ops::{Deref, DerefMut};

use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::PixelFormatEnum;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::{Texture, TextureCreator};
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::character::Direction;
use crate::system::System;
use crate::texture_holder::{TextureHolder, TextureId, Textures};
use crate::weapons::{Nothing, Sword};
use crate::{GetDimension, GetPos, ONE_SECOND};

#[allow(dead_code)]
pub enum WeaponKind {
    Nothing(Nothing),
    Sword(Sword),
    LongSword,
    Axe,
    Arc,
    Mass,
    Hammer,
    Spear,
    Dagger,
    Wand,
}

pub struct Weapon {
    pub x: i64,
    pub y: i64,
    pub action: Option<WeaponAction>,
    pub blocking_direction: Option<Direction>,
    pub kind: WeaponKind,
    pub data_id: &'static str,
    /// Total time required for this weapon to perform its action.
    pub total_time: u32,
    pub attack: i32,
}

impl Weapon {
    /// Returns a vec of positions to check.
    // FIXME: This whole thing is terrible performance-wise...
    pub fn compute_angle(&self, textures: &Textures<'_>) -> Option<Vec<(i64, i64)>> {
        let action = self.get_action()?;
        let height = self.height() as i64;
        let width = self.width() as i64;
        let half_width = width / 2;
        let data = textures.get_data(self.data_id);
        let mut matrix = Vec::with_capacity(data.len());

        let radian_angle = action.angle as f32 * 0.0174533;
        let radian_sin = radian_angle.sin();
        let radian_cos = radian_angle.cos();
        for y in 0..height {
            let y = (height - 1 - y) as usize;
            let y_f32 = y as f32;
            let rad_sin_y = y_f32 * radian_sin;
            let rad_cos_y = y_f32 * radian_cos;
            for x in 0..width {
                if unsafe { *data.get_unchecked(y * width as usize + x as usize) } == 0 {
                    continue;
                }
                let x = x - half_width;
                let x2 = x as f32 * radian_cos - rad_sin_y;
                let y2 = x as f32 * radian_sin + rad_cos_y;
                matrix.push((self.x - x2 as i64, self.y - y2 as i64 + action.y_add as i64));
            }
        }
        Some(matrix)
    }
    /// Set the position based on the character and its direction.
    pub fn set_pos(&mut self, x: i64, y: i64) {
        self.x = x;
        self.y = y;
    }
    pub fn update(&mut self, elapsed: u32) {
        if self.blocking_direction.is_some() {
            return;
        }
        if let Some(mut action) = self.action.take() {
            if action.duration > elapsed {
                action.duration -= elapsed;

                let angle_add = elapsed * action.total_angle / self.total_time;

                action.angle += angle_add as i32;
                self.action = Some(action);
            }
        }
    }
    pub fn draw(&self, system: &mut System, _debug: bool) {
        if let Some(direction) = self.blocking_direction {
            if let Some(texture) = self.get_texture() {
                let x = self.x - system.x();
                let y = self.y - system.y();
                let (angle, _, _) = match direction {
                    Direction::Up => (90, 0, self.height() as i32),
                    Direction::Right => (180, self.width() as i32, 0),
                    Direction::Down => (270, self.width() as i32, self.height() as i32),
                    Direction::Left => (0, 0, 0),
                };
                system.copy_ex_to_canvas(
                    texture,
                    None,
                    Rect::new(x as i32, y as i32, self.width(), self.height()),
                    angle as f64,
                    None,
                    false,
                    false,
                );
            }
        } else if let Some(ref action) = self.action {
            if let Some(texture) = self.get_texture() {
                let x = self.x - system.x();
                let y = self.y - system.y();
                system.copy_ex_to_canvas(
                    texture,
                    None,
                    Rect::new(x as i32, y as i32, self.width(), self.height()),
                    action.angle as f64,
                    Some((action.x_add, action.y_add).into()),
                    false,
                    false,
                );
            }
        }
    }
    pub fn is_blocking(&self) -> bool {
        self.blocking_direction.is_some()
    }
    pub fn block(&mut self, direction: Direction) {
        self.blocking_direction = Some(direction);
        self.action = None;
    }
    pub fn stop_block(&mut self) {
        self.blocking_direction = None;
    }
    pub fn is_attacking(&self) -> bool {
        self.action.is_some()
    }
    pub fn get_action(&self) -> Option<&WeaponAction> {
        self.action.as_ref()
    }
    pub fn stop_use(&mut self) {
        self.action = None;
    }
    pub fn use_it(&mut self, direction: Direction) {
        self.action = self.kind.use_it(direction);
        self.blocking_direction = None;
    }
}

impl GetPos for Weapon {
    fn x(&self) -> i64 {
        self.x
    }
    fn y(&self) -> i64 {
        self.y
    }
}

impl GetDimension for Weapon {
    fn width(&self) -> u32 {
        self.kind.width()
    }
    fn height(&self) -> u32 {
        self.kind.height()
    }
}

impl Deref for Weapon {
    type Target = WeaponKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl DerefMut for Weapon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl WeaponKind {
    fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        match *self {
            Self::Nothing(ref mut n) => n.use_it(direction),
            Self::Sword(ref mut s) => s.use_it(direction),
            _ => None,
        }
    }
    pub fn weight(&self) -> u32 {
        match *self {
            Self::Nothing(ref s) => s.weight(),
            Self::Sword(ref s) => s.weight(),
            _ => 0,
        }
    }
    pub fn get_texture(&self) -> Option<TextureId> {
        match *self {
            Self::Nothing(ref n) => n.get_texture(),
            Self::Sword(ref s) => s.get_texture(),
            _ => None,
        }
    }
}

impl GetDimension for WeaponKind {
    fn width(&self) -> u32 {
        match *self {
            Self::Nothing(ref n) => n.width(),
            Self::Sword(ref s) => s.width(),
            _ => 0,
        }
    }
    fn height(&self) -> u32 {
        match *self {
            Self::Nothing(ref n) => n.height(),
            Self::Sword(ref s) => s.height(),
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct WeaponAction {
    pub angle: i32,
    /// The angle of the rotation.
    pub total_angle: u32,
    pub duration: u32,
    pub x_add: i32,
    pub y_add: i32,
}
