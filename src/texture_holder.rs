use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::{Color, PixelFormatEnum};
use crate::sdl2::rect::Rect;
use crate::sdl2::render::{Texture, TextureCreator};
use crate::sdl2::surface::Surface;
use crate::sdl2::ttf::Font;
use crate::sdl2::video::WindowContext;

use crate::system::System;

use std::collections::HashMap;

pub struct TextureHolder<'a> {
    pub texture: Texture<'a>,
    pub width: u32,
    pub height: u32,
}

impl<'a> TextureHolder<'a> {
    pub fn surface_into_texture(
        texture_creator: &'a TextureCreator<WindowContext>,
        surface: Surface,
    ) -> TextureHolder<'a> {
        Self::surface_to_texture(texture_creator, &surface)
    }

    pub fn surface_to_texture(
        texture_creator: &'a TextureCreator<WindowContext>,
        surface: &Surface,
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
        Self::surface_into_texture(
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
        Self::surface_into_texture(texture_creator, surface)
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

#[derive(Clone, Copy)]
pub struct TextureId {
    id: usize,
    pub height: u16,
    pub width: u16,
}

impl TextureId {
    fn new(id: usize, height: u32, width: u32) -> Self {
        Self {
            id,
            height: height as _,
            width: width as _,
        }
    }

    pub fn draw(self, system: &mut System, x: i64, y: i64) {
        let texture = system.textures.get(self);
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
                &texture.texture,
                None,
                Rect::new(x, y, self.width as _, self.height as _),
            )
            .expect("failed to draw texture from texture holder");
    }
}

pub struct Textures<'a> {
    textures: Vec<TextureHolder<'a>>,
    named_textures: HashMap<&'static str, TextureId>,
    surface_data: HashMap<&'static str, Vec<u8>>,
    surfaces: HashMap<&'static str, Surface<'a>>,
}

impl<'a> Textures<'a> {
    pub fn new() -> Self {
        Self {
            textures: Vec::with_capacity(10),
            named_textures: HashMap::new(),
            surface_data: HashMap::new(),
            surfaces: HashMap::new(),
        }
    }

    pub fn add_texture(&mut self, texture: TextureHolder<'a>) -> TextureId {
        let width = texture.width;
        let height = texture.height;
        self.textures.push(texture);
        TextureId::new(self.textures.len() - 1, height, width)
    }

    pub fn add_named_texture(&mut self, name: &'static str, texture: TextureHolder<'a>) {
        let id = self.add_texture(texture);
        self.create_named_texture_id(name, id);
    }

    pub fn add_texture_from_image(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        img_path: &str,
    ) -> TextureId {
        self.add_texture(TextureHolder::from_image(texture_creator, img_path))
    }

    pub fn create_named_texture_from_image(
        &mut self,
        name: &'static str,
        texture_creator: &'a TextureCreator<WindowContext>,
        img_path: &str,
    ) {
        let id = self.add_texture_from_image(texture_creator, img_path);
        self.create_named_texture_id(name, id);
    }

    pub fn create_named_texture_id(&mut self, name: &'static str, id: TextureId) {
        assert!(
            self.named_textures.insert(name, id).is_none(),
            "Duplicated ID for texture {}",
            name
        );
    }

    pub fn get_texture_id_from_name(&self, name: &str) -> TextureId {
        *self
            .named_textures
            .get(&name)
            .expect("tried to get unknown texture")
    }

    #[inline]
    pub fn get(&self, id: TextureId) -> &TextureHolder<'a> {
        unsafe { self.textures.get_unchecked(id.id) }
    }

    pub fn add_surface_data(&mut self, name: &'static str, data: Vec<u8>) {
        assert!(
            self.surface_data.insert(name, data).is_none(),
            "Duplicated ID for surface data {}",
            name
        );
    }

    #[inline]
    pub fn get_data(&self, name: &'static str) -> &[u8] {
        self.surface_data
            .get(&name)
            .expect("tried to get unknown surface data")
    }

    pub fn add_surface(&mut self, name: &'static str, surface: Surface<'a>) {
        assert!(
            self.surfaces.insert(name, surface).is_none(),
            "Duplicated ID for surface {}",
            name
        );
    }

    #[inline]
    pub fn get_surface(&self, id: &'static str) -> &Surface<'a> {
        self.surfaces.get(id).expect("tried to get unknown surface")
    }
}
