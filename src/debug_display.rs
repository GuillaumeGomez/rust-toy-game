use crate::sdl2::pixels::{Color, PixelFormatEnum};
use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::{TextureHolder, TextureId, Textures};
use crate::{GetDimension, GetPos, HEIGHT, MAP_CASE_SIZE, WIDTH};

pub struct DebugDisplay {
    background: TextureId,
    width: u32,
    height: u32,
    font_size: u16,
    draw_grid: bool,
    grid: TextureId,
}

impl DebugDisplay {
    pub fn new<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
        font_size: u16,
    ) -> Self {
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
            background: textures.add_texture(TextureHolder::surface_into_texture(
                texture_creator,
                background,
            )),
            width,
            height,
            font_size,
            grid: textures.add_texture(TextureHolder::surface_into_texture(texture_creator, grid)),
            draw_grid: false,
        }
    }

    pub fn switch_draw_grid(&mut self) {
        self.draw_grid = !self.draw_grid;
    }

    pub fn draw(&self, system: &mut System, text: &str) {
        if self.draw_grid {
            let x_add = system.x().abs() % MAP_CASE_SIZE as f32;
            let y_add = system.y().abs() % MAP_CASE_SIZE as f32;
            system.copy_to_canvas(
                self.grid,
                Rect::new(
                    (MAP_CASE_SIZE as f32 - x_add) as i32,
                    (MAP_CASE_SIZE as f32 - y_add) as i32,
                    system.width() as u32,
                    system.height() as u32,
                ),
                None,
            );
        }
        if text.is_empty() {
            return;
        }
        system.copy_to_canvas(
            self.background,
            None,
            Rect::new(0, 0, self.width, self.height),
        );
        let mut current_pos = 2;
        for line in text.lines() {
            if !line.is_empty() {
                let (_, h) = system.draw_text(
                    line,
                    self.font_size,
                    Color::RGB(255, 255, 255),
                    3,
                    current_pos,
                    false,
                    false,
                );
                current_pos += h as i32 + 1; // 1 is for having spacing between lines
            } else {
                current_pos += self.font_size as i32 + 1; // 1 is for having spacing between lines
            }
            if current_pos as u32 > self.height {
                break;
            }
        }
        system.full_draw_text(3, current_pos);
    }
}
