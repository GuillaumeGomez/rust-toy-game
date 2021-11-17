use crate::sdl2::pixels::{Color, PixelFormatEnum};
use crate::sdl2::rect::Rect;
use crate::sdl2::render::{Texture, TextureCreator};
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::system::System;
use crate::traits::{GetDimension, GetPos};

pub struct HealthBar<'a> {
    bar: Texture<'a>,
    fill_bar: Texture<'a>,
    pub width: u32,
    pub height: u32,
}

impl<'a> HealthBar<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        width: u32,
        height: u32,
    ) -> HealthBar<'a> {
        let mut bar = Surface::new(width, height, PixelFormatEnum::RGBA8888)
            .expect("failed to create HealthBar surface");
        bar.fill_rect(None, Color::RGB(0, 0, 0))
            .expect("failed to fill HealthBar rect 1");
        bar.fill_rect(
            Rect::new(1, 1, width - 2, height - 2),
            Color::RGBA(0, 0, 0, 0),
        )
        .expect("failed to fill HealthBar rect 2");

        let mut fill_bar = Surface::new(
            width - 2,
            height - 2,
            texture_creator.default_pixel_format(),
        )
        .expect("failed to create HealthBar fill surface");
        fill_bar
            .fill_rect(None, Color::RGB(255, 0, 0))
            .expect("failed to fill HealthBar rect");

        HealthBar {
            bar: texture_creator
                .create_texture_from_surface(bar)
                .expect("failed to build texture from HealthBar surface"),
            fill_bar: texture_creator
                .create_texture_from_surface(fill_bar)
                .expect("failed to build texture from HealthBar fill surface"),
            width,
            height,
        }
    }

    pub fn draw(&self, x: f32, y: f32, pourcent: u32, system: &mut System) {
        let x = (x - system.x()) as i32;
        let y = (y - system.y()) as i32;
        if x + self.width as i32 >= 0
            && x < system.width() as i32
            && y + self.height as i32 >= 0
            && y < system.height() as i32
        {
            system
                .canvas
                .copy(&self.bar, None, Rect::new(x, y, self.width, self.height))
                .expect("copy HealthBar failed");
            let width = (self.width - 2) * pourcent / 100;
            if width > 0 {
                system
                    .canvas
                    .copy(
                        &self.fill_bar,
                        Rect::new(0, 0, width, self.height - 2),
                        Rect::new(x + 1, y + 1, width, self.height - 2),
                    )
                    .expect("copy HealthBar fill failed");
            }
        }
    }
}
