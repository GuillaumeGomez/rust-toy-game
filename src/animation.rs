use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::{TextureId, Textures};
use crate::ONE_SECOND;

const DEATH_SPRITE_WIDTH: u32 = 30;
const DEATH_SPRITE_HEIGHT: u32 = 22;
const LEVEL_UP_SPRITE_WIDTH: u32 = 82;
const LEVEL_UP_SPRITE_HEIGHT: u32 = 36;

pub fn create_death_animation_texture<'a>(
    textures: &mut Textures<'a>,
    texture_creator: &'a TextureCreator<WindowContext>,
) {
    textures.create_named_texture_from_image("death", texture_creator, "resources/death.png");
}

pub fn create_level_up_animation_texture<'a>(
    textures: &mut Textures<'a>,
    texture_creator: &'a TextureCreator<WindowContext>,
) {
    textures.create_named_texture_from_image("level-up", texture_creator, "resources/level-up.png");
}

pub struct Animation {
    pub texture: TextureId,
    nb_animations: u32,
    duration: u32,
    max_duration: u32,
    sprite_width: u32,
    sprite_height: u32,
    pub sprite_display_width: u32,
    pub sprite_display_height: u32,
}

impl Animation {
    pub fn new_death(textures: &Textures<'_>) -> Self {
        let texture = textures.get_texture_id_from_name("death");
        let nb_animations = textures.get(texture).width / DEATH_SPRITE_WIDTH;
        Animation {
            texture,
            nb_animations,
            duration: 0,
            max_duration: ONE_SECOND,
            sprite_width: DEATH_SPRITE_WIDTH,
            sprite_height: DEATH_SPRITE_HEIGHT,
            sprite_display_width: DEATH_SPRITE_WIDTH,
            sprite_display_height: DEATH_SPRITE_HEIGHT,
        }
    }

    pub fn new_level_up(textures: &Textures<'_>) -> Self {
        let texture = textures.get_texture_id_from_name("level-up");
        let nb_animations = textures.get(texture).width / LEVEL_UP_SPRITE_WIDTH;
        Animation {
            texture,
            nb_animations,
            duration: 0,
            max_duration: ONE_SECOND,
            sprite_width: LEVEL_UP_SPRITE_WIDTH,
            sprite_height: LEVEL_UP_SPRITE_HEIGHT,
            sprite_display_width: 38,
            sprite_display_height: 16,
        }
    }

    pub fn update(&mut self, elapsed: u32) {
        self.duration += elapsed;
    }

    pub fn draw(&self, system: &mut System, x: i64, y: i64) {
        if self.is_done() {
            return;
        }
        let x = (x - system.x()) as i32 - (self.sprite_display_width / 2) as i32;
        let y = (y - system.y()) as i32 - (self.sprite_display_height / 2) as i32;

        if self.sprite_display_width as i32 + x < 0
            || x > system.width()
            || self.sprite_display_height as i32 + y < 0
            || y > system.height()
        {
            return;
        }
        let current_animation =
            (self.duration as u32 * 100 / self.max_duration as u32) * self.nb_animations / 100;
        let tile_x = current_animation * self.sprite_width as u32;
        system.copy_to_canvas(
            self.texture,
            Rect::new(tile_x as i32, 0, self.sprite_width, self.sprite_height),
            Rect::new(x, y, self.sprite_display_width, self.sprite_display_height),
        );
    }

    pub fn is_done(&self) -> bool {
        self.duration >= self.max_duration
    }
}
