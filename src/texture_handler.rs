use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::surface::Surface;

use std::ops::{Deref, DerefMut};

use crate::character::Direction;

#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    rect: Rect,
    pub incr_to_next: i32,
}

impl Dimension {
    pub fn new(rect: Rect, incr_to_next: i32) -> Dimension {
        Dimension { rect, incr_to_next }
    }
}

impl Deref for Dimension {
    type Target = Rect;

    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

impl DerefMut for Dimension {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rect
    }
}

pub struct TextureHandler<'a> {
    /// We keep this surface for collisions check (it's way too slow to do it on a texture!).
    surface: &'a Surface<'a>,
    pub texture: &'a Texture<'a>,
    pub actions_standing: Vec<Dimension>,
    /// The second element is the number of "animations".
    pub actions_moving: Vec<(Dimension, i32)>,
    pub forced_size: Option<(u32, u32)>,
}

impl<'a> TextureHandler<'a> {
    pub fn new(
        surface: &'a Surface<'a>,
        texture: &'a Texture<'a>,
        actions_standing: Vec<Dimension>,
        actions_moving: Vec<(Dimension, i32)>,
        forced_size: Option<(u32, u32)>,
    ) -> TextureHandler<'a> {
        // if let Some((width, height)) = forced_size {
        //     let mut forced_surface = Surface::new(width * 3, height * 4, surface.pixel_format_enum()).expect("failed to create new surface for resize");
        //     surface.blit(None, &mut forced_surface, Rect::new(0, 0, width * 3, height * 4)).expect("failed to resize surface...");
        //     TextureHandler {
        //         surface,
        //         texture,
        //         actions_standing,
        //         actions_moving,
        //         forced_surface: Some(forced_surface),
        //         forced_size,
        //     }
        // } else {
        TextureHandler {
            surface,
            texture,
            actions_standing,
            actions_moving,
            // forced_surface: None,
            forced_size,
        }
        // }
    }

    pub fn check_intersection(
        &self,
        matrix: &[(i64, i64)],
        dir: Direction,
        is_moving: bool,
        character_pos: (i64, i64),
    ) -> bool {
        // let (surface, tile_pos) = match self.forced_surface {
        //     Some(ref s) => {
        //         let (width, height) = self.forced_size.as_ref().unwrap();
        //         (s, (tile_pos.0 / (self.surface.size().0 / width) as i32, tile_pos.1 / (self.surface.size().1 / height) as i32))
        //     }
        //     None => (self.surface, tile_pos),
        // };
        let (mut tile_pos_width, mut tile_pos_height) = if is_moving {
            let tmp = &self.actions_moving[dir as usize].0;
            (tmp.width() as i32, tmp.height() as i32)
        } else {
            let tmp = &self.actions_standing[dir as usize];
            (tmp.width() as i32, tmp.height() as i32)
        };
        let tile_size = match self.forced_size {
            Some(s) => {
                tile_pos_width /= self.surface.size().0 as i32 / s.0 as i32;
                tile_pos_height /= self.surface.size().1 as i32 / s.1 as i32;
                s
            }
            None => self.surface.size(),
        };
        let pitch = self.surface.pitch() as i32;
        let max_len = (self.surface.height() * self.surface.pitch()) as i32;
        let surface = self.surface.raw();
        let pixels = unsafe { (*surface).pixels as *const u8 };
        for (x, y) in matrix.iter() {
            let x = (x - character_pos.0) as i32 + tile_pos_width;
            let y = (y - character_pos.1) as i32 + tile_pos_height;
            if y < tile_pos_height
                || y > tile_pos_height + tile_size.1 as i32
                || x < tile_pos_width
                || x > tile_pos_width + tile_size.0 as i32
            {
                // We are outside of the tile we're looking for!
                continue;
            }
            let pos = y * pitch + x * 4; // 4 is because the surfaces are always RGBA8888 so 4 bytes
            if pos >= 0 && pos < max_len {
                let target_pixel = unsafe { *(pixels.add(pos as usize) as *const u32) };
                let alpha = target_pixel & 255;
                if alpha > 220 {
                    // We consider something with an alpha to more than 86% to be part of the character.
                    return true;
                }
            }
        }
        false
    }
}
