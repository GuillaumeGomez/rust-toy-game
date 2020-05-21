use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::surface::Surface;

use std::ops::Deref;

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

pub struct TextureHandler<'a> {
    /// We keep this surface for collisions check (it's way too slow to do it on a texture!).
    surface: &'a Surface<'a>,
    pub texture: &'a Texture<'a>,
    pub actions_standing: Vec<Dimension>,
    /// The second element is the number of "animations".
    pub actions_moving: Vec<(Dimension, i32)>,
}

impl<'a> TextureHandler<'a> {
    pub fn new(
        surface: &'a Surface<'a>,
        texture: &'a Texture<'a>,
        actions_standing: Vec<Dimension>,
        actions_moving: Vec<(Dimension, i32)>,
    ) -> TextureHandler<'a> {
        TextureHandler {
            surface,
            texture,
            actions_standing,
            actions_moving,
        }
    }

    pub fn check_intersection(
        &self,
        matrix: &[(i64, i64)],
        tile_pos: (i32, i32),
        tile_size: (i32, i32),
        character_pos: (i64, i64),
    ) -> bool {
        let pitch = self.surface.pitch() as i32;
        let max_len = (self.surface.height() * self.surface.pitch()) as i32;
        let surface = self.surface.raw();
        let pixels = unsafe { (*surface).pixels as *const u8 };
        for (x, y) in matrix.iter() {
            let x = (x - character_pos.0) as i32 + tile_pos.0;
            let y = (y - character_pos.1) as i32 + tile_pos.1;
            if y < tile_pos.1
                || y > tile_pos.1 + tile_size.1
                || x < tile_pos.0
                || x > tile_pos.0 + tile_size.0
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
