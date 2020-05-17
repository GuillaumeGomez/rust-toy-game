use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;

const DEATH_SPRITE_WIDTH: u32 = 30;
const DEATH_SPRITE_HEIGHT: u32 = 22;

pub struct DeathAnimation<'a> {
    texture: Texture<'a>,
    nb_animations: u32,
    duration: u64,
    max_duration: u64,
}

impl<'a> DeathAnimation<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        duration: u64,
    ) -> DeathAnimation<'a> {
        let mut surface = Surface::from_file("resources/death.png")
            .expect("failed to load `resources/death.png`");
        DeathAnimation {
            texture: texture_creator
                .create_texture_from_surface(&surface)
                .expect("failed to build texture from surface"),
            nb_animations: surface.width() / DEATH_SPRITE_WIDTH,
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
        let x = (x - system.x()) as i32 - DEATH_SPRITE_WIDTH as i32 / 2;
        let y = (y - system.y()) as i32 - DEATH_SPRITE_HEIGHT as i32 / 2;

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
                &self.texture,
                Rect::new(tile_x as i32, 0, DEATH_SPRITE_WIDTH, DEATH_SPRITE_HEIGHT),
                Rect::new(
                    x + DEATH_SPRITE_WIDTH as i32 / 2,
                    y + DEATH_SPRITE_HEIGHT as i32 / 2,
                    DEATH_SPRITE_WIDTH,
                    DEATH_SPRITE_HEIGHT,
                ),
            )
            .expect("copy death failed");
    }

    pub fn is_done(&self) -> bool {
        self.duration >= self.max_duration
    }
}
