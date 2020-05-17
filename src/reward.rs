use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::TextureHolder;

pub struct RewardInfo {
    pub gold: u32,
}

pub struct Reward<'a> {
    texture: &'a TextureHolder<'a>,
    text: &'a TextureHolder<'a>,
    x: i64,
    y: i64,
    info: RewardInfo,
}

impl<'a> Reward<'a> {
    pub fn new(
        texture: &'a TextureHolder<'a>,
        text: &'a TextureHolder<'a>,
        x: i64,
        y: i64,
        info: RewardInfo,
    ) -> Reward<'a> {
        Reward {
            texture,
            text,
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
        // TODO: make it conditional only when the player is looking at it (so only one at a time!)
        system
            .canvas
            .copy(
                &self.text.texture,
                None,
                Rect::new(
                    x + (self.texture.width as i32) / 2 - (self.text.width as i32) / 2,
                    y - 2 - self.text.height as i32,
                    self.text.width,
                    self.text.height,
                ),
            )
            .expect("copy reward failed");
    }
}
