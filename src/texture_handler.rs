use crate::sdl2::rect::Rect;
use crate::sdl2::render::Texture;
use crate::sdl2::surface::Surface;

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
        let (mut tile_x, mut tile_y, mut tile_width, mut tile_height) = if is_moving {
            let tmp = &self.actions_moving[dir as usize].0;
            (tmp.x(), tmp.y(), tmp.width() as i32, tmp.height() as i32)
        } else {
            let tmp = &self.actions_standing[dir as usize];
            (tmp.x(), tmp.y(), tmp.width() as i32, tmp.height() as i32)
        };
        if let Some(s) = self.forced_size {
            tile_width = s.0 as i32;
            tile_height = s.1 as i32;
            tile_x /= self.surface.size().0 as i32 / s.0 as i32;
            tile_y = self.surface.size().1 as i32 / s.1 as i32;
        }
        let pitch = self.surface.pitch() as i32;
        let max_len = (self.surface.height() * self.surface.pitch()) as i32;
        let surface = self.surface.raw();
        let pixels = unsafe { (*surface).pixels as *const u8 };
        for (x, y) in matrix.iter() {
            let x = (x - character_pos.0) as i32 + tile_x;
            let y = (y - character_pos.1) as i32 + tile_y;
            if y < tile_y || y > tile_y + tile_height || x < tile_x || x > tile_x + tile_width {
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
