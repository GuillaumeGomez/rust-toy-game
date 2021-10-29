use std::ops::{Deref, DerefMut};

use crate::sdl2::rect::Rect;

use parry2d::shape::ConvexPolygon;
use parry2d::math::Point;

use crate::character::Direction;
use crate::system::System;
use crate::texture_holder::{TextureId, Textures};
use crate::weapons::{Nothing, Sword};
use crate::{GetDimension, GetPos};

#[allow(dead_code)]
#[derive(Debug)]
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

#[derive(Debug)]
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
        action: &Option<WeaponAction>,
    ) -> Option<ConvexPolygon> {
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
        let height = self.height() as i32;
        let width = self.width() as i32;
        let half_width = width / 2;

        let current_angle = start_angle
            + action.duration as f32 / action.total_duration as f32 * total_angle;
        let radian_angle = current_angle * 0.0174533;
        let radian_sin = radian_angle.sin();
        let radian_cos = radian_angle.cos();

        // FIXME: replace with actual hitbox.
        let mut poses = [
            Point::new(0., 0.),
            Point::new(width as f32, 0.),
            Point::new(width as f32, height as f32),
            Point::new(0., height as f32),
        ];
        for ref mut p in &mut poses {
            let y = (height - 1) as f32 - p.y;
            let rad_sin_y = y * radian_sin;
            let rad_cos_y = y * radian_cos;
            let x = p.x - half_width as f32;
            p.x = self.x as f32 - (x * radian_cos - rad_sin_y);
            p.y = self.y as f32 - (x * radian_sin + rad_cos_y) + action.y_add as f32;
        }
        ConvexPolygon::from_convex_hull(&poses)
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
                    let current_angle = (start_angle
                        + action.duration as f32 / action.total_duration as f32
                            * total_angle) as f64;
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
        !matches!(self.kind, WeaponKind::Nothing(_))
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

impl WeaponAction {
    pub fn get_attack_by_move_target(&self) -> Option<(i32, i32)> {
        match self.kind {
            WeaponActionKind::AttackByMove { target_x, target_y } => {
                // We consider that the target must be reached in half the time and then go back to
                // where it's supposed to be.
                let (x_add, y_add) = if self.duration > self.total_duration / 2 {
                    (
                        target_x - target_x * self.duration as i32 / self.total_duration as i32,
                        target_y - target_y * self.duration as i32 / self.total_duration as i32,
                    )
                } else {
                    (
                        target_x * self.duration as i32 / self.total_duration as i32,
                        target_y * self.duration as i32 / self.total_duration as i32,
                    )
                };
                Some((x_add, y_add))
            }
            _ => None,
        }
    }

    pub fn is_attack_by_move(&self) -> bool {
        matches!(self.kind, WeaponActionKind::AttackByMove { .. })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeaponActionKind {
    /// Sword, Mass, Hammer...
    AttackBySlash {
        start_angle: f32,
        /// The total angle of the rotation that the attack will do.
        total_angle: f32,
    },
    /// Magic staff, bow...
    AttackByProjection,
    /// When you don't have a weapon and use your body instead...
    AttackByMove {
        // (x, y) of where the move will end.
        target_x: i32,
        target_y: i32,
    },
}
