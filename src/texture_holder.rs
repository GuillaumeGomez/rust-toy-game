use sdl2::image::LoadSurface;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::system::System;

pub struct TextureHolder<'a> {
    pub texture: Texture<'a>,
    pub width: u32,
    pub height: u32,
}

impl<'a> TextureHolder<'a> {
    fn surface_to_texture(
        texture_creator: &'a TextureCreator<WindowContext>,
        surface: Surface,
    ) -> TextureHolder<'a> {
        let width = surface.width();
        let height = surface.height();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .expect("failed to build texture from surface");

        TextureHolder {
            texture,
            width,
            height,
        }
    }

    pub fn from_image(
        texture_creator: &'a TextureCreator<WindowContext>,
        img_path: &str,
    ) -> TextureHolder<'a> {
        Self::surface_to_texture(
            texture_creator,
            Surface::from_file(img_path)
                .map_err(|err| format!("failed to load `{}`: {}", img_path, err))
                .unwrap(),
        )
    }

    pub fn from_text(
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'a Font,
        color: Color,
        wrap_color: Option<Color>,
        text: &str,
    ) -> TextureHolder<'a> {
        macro_rules! update_color {
            ($data:ident, $x:expr, $y:expr, $wrap_color:ident, $target_color:ident) => {{
                let pos = $x + $y;
                let prev = ($data[pos] as u32) << 16
                    | ($data[pos + 1] as u32) << 8
                    | $data[pos + 2] as u32;
                if prev != $target_color {
                    $data[pos] = $wrap_color.r;
                    $data[pos + 1] = $wrap_color.g;
                    $data[pos + 2] = $wrap_color.b;
                    $data[pos + 3] = 255;
                }
            }};
        }
        let mut surface = font
            .render(text)
            .blended(color)
            .expect("failed to convert text to surface");
        // TODO: might be nice to fix it so that the text is visible in any context!
        if let Some(wrap_color) = wrap_color {
            if surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
                surface = surface
                    .convert_format(PixelFormatEnum::RGBA8888)
                    .expect("failed to convert surface to RGBA8888");
            }

            let target_color = ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32;
            let pitch = surface.pitch() as usize;
            let width = surface.width() as usize;
            let height = surface.height() as usize;
            surface.with_lock_mut(|data| {
                for y in 0..height {
                    let y_pitch = y * pitch;
                    for x in 0..width {
                        let x_pos = x * 4;
                        let pos = y_pitch + x_pos;
                        let target_pixel = (data[pos] as u32) << 16
                            | (data[pos + 1] as u32) << 8
                            | data[pos + 2] as u32;
                        if target_pixel == target_color {
                            if x > 0 {
                                update_color!(data, x_pos - 4, y_pitch, wrap_color, target_color);
                            }
                            if x + 1 < width {
                                update_color!(data, x_pos + 4, y_pitch, wrap_color, target_color);
                            }
                            if y > 0 {
                                update_color!(
                                    data,
                                    x_pos,
                                    (y - 1) * pitch,
                                    wrap_color,
                                    target_color
                                );
                            }
                            if y + 1 < height {
                                update_color!(
                                    data,
                                    x_pos,
                                    (y + 1) * pitch,
                                    wrap_color,
                                    target_color
                                );
                            }
                        }
                    }
                }
            });
        }
        Self::surface_to_texture(texture_creator, surface)
    }

    pub fn with_max_size(mut self, max_size: u32) -> TextureHolder<'a> {
        if self.width > self.height {
            let div = self.width / max_size;
            self.width = max_size;
            self.height = self.height / div;
            if self.height > max_size {
                self.height = max_size;
            }
        } else {
            let div = self.height / max_size;
            self.height = max_size;
            self.width = self.width / div;
            if self.width > max_size {
                self.width = max_size;
            }
        }
        self
    }

    pub fn draw(&self, system: &mut System, x: i64, y: i64) {
        let x = (x - system.x()) as i32;
        let y = (y - system.y()) as i32;

        if self.width as i32 + x < 0
            || x > system.width()
            || self.height as i32 + y < 0
            || y > system.height()
        {
            return;
        }
        system
            .canvas
            .copy(
                &self.texture,
                None,
                Rect::new(x, y, self.width, self.height),
            )
            .expect("failed to draw texture from texture holder");
    }
}
