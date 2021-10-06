use crate::sdl2::pixels::Color;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::{BlendMode, TextureCreator};
use crate::sdl2::surface::Surface;
use crate::sdl2::ttf::Font;
use crate::sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::TextureHolder;

struct CharInfo {
    c: char,
    x: i32,
    height: u32,
    width: u32,
}

pub struct FontHandler<'a> {
    pub texture: TextureHolder<'a>,
    pub size: u16,
    pub color: Color,
    inner: Vec<CharInfo>,
}

impl<'a> FontHandler<'a> {
    pub fn new<'b>(
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'b Font<'b, 'static>,
        font_size: u16,
        color: Color,
    ) -> FontHandler<'a> {
        let mut max_height = 0;
        let mut current_width = 0;

        let v = "ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz0123456789ÇÀÉÈÊÙÛçàéêùû#(){}[]_&+-*/%='\",;.?:!"
            .chars()
            .map(|c| {
                let surface = font
                    .render(&c.to_string())
                    .blended(color)
                    .expect("failed to convert letter to surface");
                if surface.height() > max_height {
                    max_height = surface.height();
                }
                current_width += surface.width();
                (c, surface)
            })
            .collect::<Vec<_>>();
        let mut letters_surface =
            Surface::new(current_width, max_height, v[0].1.pixel_format_enum())
                .expect("Failed to create surface for font map");
        current_width = 0;
        let inner = v
            .into_iter()
            .map(|(c, mut surface)| {
                // If we don't set this blend mode, the letters won't render nicely...
                surface
                    .set_blend_mode(BlendMode::None)
                    .expect("cannot set blend mode...");
                surface
                    .blit(
                        None,
                        &mut letters_surface,
                        Rect::new(
                            current_width as i32,
                            max_height as i32 - surface.height() as i32,
                            surface.width(),
                            surface.height(),
                        ),
                    )
                    .expect("failed to copy letter...");
                let ret = CharInfo {
                    c,
                    x: current_width as i32,
                    height: max_height,
                    width: surface.width(),
                };
                current_width += surface.width();
                ret
            })
            .collect::<Vec<_>>();
        FontHandler {
            texture: TextureHolder::surface_to_texture(texture_creator, letters_surface),
            size: font_size,
            color,
            inner,
        }
    }

    pub fn draw(
        &self,
        system: &mut System,
        text: &str,
        x: i32,
        y: i32,
        x_centered: bool,
        y_centered: bool,
    ) -> (u32, u32) {
        let (mut x, y) = if x_centered || y_centered {
            let mut max_height = 0;
            let mut total_width = 0;
            for c in text.chars() {
                total_width += self
                    .inner
                    .iter()
                    .find(|x| x.c == c)
                    .map(|x| {
                        if x.height > max_height {
                            max_height = x.height;
                        }
                        x.width + 1
                    })
                    .unwrap_or(0);
            }
            (
                if x_centered {
                    x - total_width as i32 / 2
                } else {
                    x
                },
                if y_centered {
                    y - max_height as i32 / 2
                } else {
                    y
                },
            )
        } else {
            (x, y)
        };
        let mut total_width = 0;
        let mut max_height = 0;
        for c in text.chars() {
            if let Some(c_info) = self.inner.iter().find(|x| x.c == c) {
                system
                    .canvas
                    .copy(
                        &self.texture.texture,
                        Rect::new(c_info.x, 0, c_info.width, c_info.height),
                        Rect::new(x, y, c_info.width, c_info.height),
                    )
                    .expect("copy letter failed");
                x += c_info.width as i32 + 1;
                total_width += c_info.width + 1;
                if c_info.height > max_height {
                    max_height = c_info.height;
                }
            }
        }
        (total_width, max_height)
    }
}
