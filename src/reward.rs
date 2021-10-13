use crate::sdl2::rect::Rect;

use crate::system::System;
use crate::texture_holder::{TextureHolder, TextureId};
use crate::{GetDimension, GetPos};

#[derive(Debug)]
pub struct RewardInfo {
    pub gold: u32,
}

#[derive(Debug)]
pub struct Reward {
    texture: TextureId,
    x: i64,
    y: i64,
    info: RewardInfo,
}

impl Reward {
    pub fn new(texture: TextureId, x: i64, y: i64, info: RewardInfo) -> Reward {
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
        system.copy_to_canvas(
            self.texture,
            None,
            Rect::new(x, y, self.texture.width as _, self.texture.height as _),
        );
    }
}

impl GetPos for Reward {
    fn x(&self) -> i64 {
        self.x
    }

    fn y(&self) -> i64 {
        self.y
    }
}

impl GetDimension for Reward {
    fn width(&self) -> u32 {
        self.texture.width as _
    }
    fn height(&self) -> u32 {
        self.texture.height as _
    }
}
