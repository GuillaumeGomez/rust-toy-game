use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use crate::WIDTH;

pub struct DebugDisplay<'a, 'b> {
    font: &'a Font<'a, 'static>,
    texture_creator: &'b TextureCreator<WindowContext>,
    background: Texture<'b>,
    width: u32,
    height: u32,
    font_size: i32,
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
        DebugDisplay {
            background: texture_creator
                .create_texture_from_surface(background)
                .expect("failed to build texture from debug surface"),
            font,
            texture_creator,
            width,
            height,
            font_size,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, text: &str) {
        canvas
            .copy(
                &self.background,
                None,
                Rect::new(0, 0, self.width, self.height),
            )
            .expect("copy failed for debug background");
        if text.is_empty() {
            return;
        }
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
                canvas
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
