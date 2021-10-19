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
    pub kind: WeaponKind,
    pub data_id: &'static str,
    /// Total time required for this weapon to perform its action.
    pub total_time: u32,
    pub attack: i32,
}

impl Weapon {
    /// Returns a vec of positions to check.
    // FIXME: This whole thing is terrible performance-wise...
    pub fn compute_angle(
        &self,
        textures: &Textures<'_>,
        action: &Option<WeaponAction>,
    ) -> Option<Vec<(i64, i64)>> {
        let action = match action {
            Some(a) => a,
            None => return None,
        };
        let (start_angle, total_angle) = match action.kind {
            WeaponActionKind::AttackBySlash {
                start_angle,
                total_angle,
            } => (start_angle, total_angle),
            _ => return None,
        };
        let height = self.height() as i64;
        let width = self.width() as i64;
        let half_width = width / 2;
        let data = textures.get_data(self.data_id);
        let mut matrix = Vec::with_capacity(data.len());

        let radian_angle = (start_angle as f32
            + action.duration as f32 / action.total_duration as f32 * total_angle as f32)
            * 0.0174533;
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
    pub fn draw_blocking(&self, system: &mut System, direction: Direction) {
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
    }
    pub fn draw(&self, system: &mut System, action: &WeaponAction) {
        if let Some(texture) = self.get_texture() {
            let x = self.x - system.x();
            let y = self.y - system.y();
            match action.kind {
                WeaponActionKind::AttackBySlash {
                    start_angle,
                    total_angle,
                } => {
                    let current_angle = (start_angle as f32
                        + action.duration as f32 / action.total_duration as f32
                            * total_angle as f32) as f64;
                    system.copy_ex_to_canvas(
                        texture,
                        None,
                        Rect::new(x as i32, y as i32, self.width(), self.height()),
                        current_angle,
                        Some((action.x_add, action.y_add).into()),
                        false,
                        false,
                    );
                }
                WeaponActionKind::AttackByMove { .. } => {}
                WeaponActionKind::AttackByProjection => {
                    // FIXME: it'll be the animation to send the projectile.
                    unimplemented!();
                }
            }
        }
    }
    pub fn can_block(&self) -> bool {
        // Obviously, if you don't have a weapon, you can't block.
        matches!(self.kind, WeaponKind::Nothing(_))
    }
    pub fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        self.kind.use_it(direction, self.total_time)
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
    fn use_it(&mut self, direction: Direction, total_time: u32) -> Option<WeaponAction> {
        match *self {
            Self::Nothing(ref mut n) => n.use_it(direction, total_time),
            Self::Sword(ref mut s) => s.use_it(direction, total_time),
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
    pub total_duration: u32,
    pub duration: u32,
    pub x_add: i32,
    pub y_add: i32,
    pub kind: WeaponActionKind,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeaponActionKind {
    /// Sword, Mass, Hammer...
    AttackBySlash {
        start_angle: i32,
        /// The total angle of the rotation that the attack will do.
        total_angle: i32,
    },
    /// Magic staff, bow...
    AttackByProjection,
    /// When you don't have a weapon and use your body instead...
    AttackByMove {
        /// (x, y) of where the move will end.
        target: (i32, i32),
    },
}

impl WeaponActionKind {
    pub fn get_attack_by_move_target(&self) -> Option<(i32, i32)> {
        match *self {
            Self::AttackByMove { target } => Some(target),
            _ => None,
        }
    }
}
