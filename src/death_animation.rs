use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::TextureHolder;

const DEATH_SPRITE_WIDTH: u32 = 30;
const DEATH_SPRITE_HEIGHT: u32 = 22;

pub struct DeathAnimation<'a> {
    pub texture: TextureHolder<'a>,
    nb_animations: u32,
    duration: u64,
    max_duration: u64,
}

impl<'a> DeathAnimation<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        duration: u64,
    ) -> DeathAnimation<'a> {
        let texture = TextureHolder::from_image(texture_creator, "resources/death.png");
        let nb_animations = texture.width / DEATH_SPRITE_WIDTH;
        DeathAnimation {
            texture,
            nb_animations,
            duration: 0,
            max_duration: duration,
        }
    }

    pub fn update(&mut self, elapsed: u64) {
        self.duration += elapsed;
    }

    pub fn draw(&self, system: &mut System, x: i64, y: i64) {
        if self.is_done() {
            return;
        }
        let x = (x - system.x()) as i32 - (DEATH_SPRITE_WIDTH / 2) as i32;
        let y = (y - system.y()) as i32 - (DEATH_SPRITE_HEIGHT / 2) as i32;

        if DEATH_SPRITE_WIDTH as i32 + x < 0
            || x > system.width()
            || DEATH_SPRITE_HEIGHT as i32 + y < 0
            || y > system.height()
        {
            return;
        }
        let current_animation =
            (self.duration as u32 * 100 / self.max_duration as u32) * self.nb_animations / 100;
        let tile_x = current_animation * DEATH_SPRITE_WIDTH;
        system
            .canvas
            .copy(
                &self.texture.texture,
                Rect::new(tile_x as i32, 0, DEATH_SPRITE_WIDTH, DEATH_SPRITE_HEIGHT),
                Rect::new(x, y, DEATH_SPRITE_WIDTH, DEATH_SPRITE_HEIGHT),
            )
            .expect("copy death failed");
    }

    pub fn is_done(&self) -> bool {
        self.duration >= self.max_duration
    }
}
