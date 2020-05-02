use sdl2::rect::Rect;
use sdl2::render::Texture;

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
    pub texture: Texture<'a>,
    pub actions_standing: Vec<Dimension>,
    /// The second element is the number of "animations".
    pub actions_moving: Vec<(Dimension, i32)>,
}
