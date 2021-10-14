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
use crate::weapon::{Weapon, WeaponAction, WeaponKind};
use crate::{GetDimension, GetPos, ONE_SECOND};

pub struct Nothing;

impl Nothing {
    pub fn new(textures: &Textures<'_>, attack: i32) -> Weapon {
        Weapon {
            x: 0,
            y: 0,
            action: None,
            data_id: "sword",
            total_time: ONE_SECOND as u32 / 5,
            kind: WeaponKind::Nothing(Nothing),
            attack,
            blocking_direction: None,
        }
    }
    pub fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        None
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
        0
    }
    fn height(&self) -> u32 {
        0
    }
}
