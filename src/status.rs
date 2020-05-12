use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::ONE_SECOND;

const STATUS_UPDATE_TIME: u64 = ONE_SECOND / 60;

pub struct Status<'a> {
    texture: Texture<'a>,
    width: i32,
    height: i32,
    // When it reaches y_limit, the status should be removed.
    y_pos: i32,
    y_limit: i32,
    duration: u64,
}

impl<'a> Status<'a> {
    pub fn new<'b>(
        font: &'b Font<'b, 'static>,
        texture_creator: &'a TextureCreator<WindowContext>,
        text: &str,
        color: Color,
    ) -> Status<'a> {
        let text_surface = font
            .render(text)
            .solid(color)
            .expect("failed to convert text to surface");
        let width = text_surface.width() as i32;
        let height = text_surface.height() as i32;
        let text_texture = texture_creator
            .create_texture_from_surface(text_surface)
            .expect("failed to build texture from debug surface");
        Status {
            texture: text_texture,
            width,
            height,
            y_pos: 0,
            y_limit: 30,
            duration: 0,
        }
    }

    pub fn update(&mut self, elapsed: u64) {
        self.duration += elapsed;
        while self.duration > STATUS_UPDATE_TIME && self.y_pos < self.y_limit {
            self.duration -= STATUS_UPDATE_TIME;
            self.y_pos += 1;
        }
    }

    pub fn draw(&self, system: &mut System, x: i64, y: i64) {
        // increase position of the text
        let x = (x - system.x()) as i32 - self.width / 2;
        let y = (y - system.y()) as i32 - self.y_pos - 10;
        if x + self.width >= 0 && x < system.width() && y + self.height >= 0 && y < system.height()
        {
            system
                .canvas
                .copy(
                    &self.texture,
                    None,
                    Rect::new(x, y, self.width as u32, self.height as u32),
                )
                .expect("copy status failed");
        }
    }

    pub fn should_be_removed(&self) -> bool {
        self.y_pos >= self.y_limit
    }
}
