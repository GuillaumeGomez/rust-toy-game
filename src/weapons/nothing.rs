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
use crate::weapon::{Weapon, WeaponAction, WeaponActionKind, WeaponKind};
use crate::{GetDimension, GetPos, ONE_SECOND};

pub struct Nothing;

pub const RANGE: i32 = 15;

impl Nothing {
    pub fn new(attack: i32) -> Weapon {
        Weapon {
            x: 0,
            y: 0,
            data_id: "",
            total_time: ONE_SECOND / 5,
            kind: WeaponKind::Nothing(Nothing),
            attack,
        }
    }
    pub fn use_it(&mut self, direction: Direction, total_duration: u32) -> Option<WeaponAction> {
        let (x, y) = match direction {
            Direction::Up => (0, -RANGE),
            Direction::Down => (0, RANGE),
            Direction::Left => (-RANGE, 0),
            Direction::Right => (RANGE, 0),
        };
        Some(WeaponAction {
            duration: 0,
            total_duration,
            x_add: 0,
            y_add: 0,
            kind: WeaponActionKind::AttackByMove { target: (x, y) },
        })
    }
    pub fn weight(&self) -> u32 {
        1
    }
    pub fn get_texture(&self) -> Option<TextureId> {
        None
    }
}

impl GetDimension for Nothing {
    fn width(&self) -> u32 {
        10
    }
    fn height(&self) -> u32 {
        10
    }
}
