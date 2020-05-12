use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::{HEIGHT, MAP_CASE_SIZE, WIDTH};

pub struct DebugDisplay<'a, 'b> {
    font: &'a Font<'a, 'static>,
    texture_creator: &'b TextureCreator<WindowContext>,
    background: Texture<'b>,
    width: u32,
    height: u32,
    font_size: i32,
    draw_grid: bool,
    grid: Texture<'b>,
}

impl<'a, 'b> DebugDisplay<'a, 'b> {
    pub fn new(
        font: &'a Font<'a, 'static>,
        texture_creator: &'b TextureCreator<WindowContext>,
        font_size: i32,
    ) -> DebugDisplay<'a, 'b> {
        let width = WIDTH as u32;
        let height = 200;
        let mut background = Surface::new(width, height, PixelFormatEnum::RGBA8888)
            .expect("failed to create debug surface");
        background
            .fill_rect(None, Color::RGBA(0, 0, 0, 150))
            .expect("failed to fill debug surface");

        // The grid used for debug.
        let mut grid = Surface::new(
            width + MAP_CASE_SIZE as u32,
            (HEIGHT + MAP_CASE_SIZE as i32) as u32,
            PixelFormatEnum::RGBA8888,
        )
        .expect("failed to create grid debug surface");
        for y in 0..HEIGHT / MAP_CASE_SIZE as i32 + 1 {
            grid.fill_rect(
                Rect::new(
                    0,
                    y * MAP_CASE_SIZE as i32,
                    (WIDTH + MAP_CASE_SIZE as i32) as u32,
                    1,
                ),
                Color::RGB(255, 0, 0),
            )
            .expect("failed to fill grid debug surface");
        }

        for x in 0..WIDTH / MAP_CASE_SIZE as i32 + 1 {
            grid.fill_rect(
                Rect::new(
                    x * MAP_CASE_SIZE as i32,
                    0,
                    1,
                    (HEIGHT + MAP_CASE_SIZE as i32) as u32,
                ),
                Color::RGB(255, 0, 0),
            )
            .expect("failed to fill grid debug surface");
        }

        DebugDisplay {
            background: texture_creator
                .create_texture_from_surface(background)
                .expect("failed to build texture from debug surface"),
            font,
            texture_creator,
            width,
            height,
            font_size,
            grid: texture_creator
                .create_texture_from_surface(grid)
                .expect("failed to build texture from grid debug surface"),
            draw_grid: false,
        }
    }

    pub fn switch_draw_grid(&mut self) {
        self.draw_grid = !self.draw_grid;
    }

    pub fn draw(&self, system: &mut System, text: &str) {
        if self.draw_grid {
            let x_add = system.x().abs() % MAP_CASE_SIZE;
            let y_add = system.y().abs() % MAP_CASE_SIZE;
            system
                .canvas
                .copy(
                    &self.grid,
                    Rect::new(
                        (MAP_CASE_SIZE - x_add) as i32,
                        (MAP_CASE_SIZE - y_add) as i32,
                        system.width() as u32,
                        system.height() as u32,
                    ),
                    None,
                )
                .expect("copy failed for grid texture");
        }
        if text.is_empty() {
            return;
        }
        system
            .canvas
            .copy(
                &self.background,
                None,
                Rect::new(0, 0, self.width, self.height),
            )
            .expect("copy failed for debug background");
        let mut current_pos = 2;
        for line in text.lines() {
            if !line.is_empty() {
                let (w, h) = self.font.size_of(line).expect("failed to get font size");
                let text_surface = self
                    .font
                    .render(line)
                    .solid(Color::RGB(255, 255, 255))
                    .expect("failed to convert text to surface");
                let text_texture = self
                    .texture_creator
                    .create_texture_from_surface(text_surface)
                    .expect("failed to build texture from debug surface");
                system
                    .canvas
                    .copy(&text_texture, None, Rect::new(3, current_pos, w, h))
                    .expect("copy failed for text texture");
                current_pos += h as i32 + 1; // 1 is for having spacing between lines
            } else {
                current_pos += self.font_size + 1; // 1 is for having spacing between lines
            }
            if current_pos as u32 > self.height {
                break;
            }
        }
    }
}
