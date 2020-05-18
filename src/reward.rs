use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::TextureHolder;
use crate::{GetDimension, GetPos};

pub struct RewardInfo {
    pub gold: u32,
}

pub struct Reward<'a> {
    texture: &'a TextureHolder<'a>,
    x: i64,
    y: i64,
    info: RewardInfo,
}

impl<'a> Reward<'a> {
    pub fn new(texture: &'a TextureHolder<'a>, x: i64, y: i64, info: RewardInfo) -> Reward<'a> {
        Reward {
            texture,
            x,
            y,
            info,
        }
    }

    pub fn draw(&self, system: &mut System) {
        let x = (self.x - system.x()) as i32;
        let y = (self.y - system.y()) as i32;

        if self.texture.width as i32 + x < 0
            || x > system.width()
            || self.texture.height as i32 + y < 0
            || y > system.height()
        {
            return;
        }
        system
            .canvas
            .copy(
                &self.texture.texture,
                None,
                Rect::new(x, y, self.texture.width, self.texture.height),
            )
            .expect("copy reward failed");
    }
}

impl<'a> GetPos for Reward<'a> {
    fn x(&self) -> i64 {
        self.x
    }

    fn y(&self) -> i64 {
        self.y
    }
}

impl<'a> GetDimension for Reward<'a> {
    fn width(&self) -> u32 {
        self.texture.width
    }
    fn height(&self) -> u32 {
        self.texture.height
    }
}
