use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

use std::collections::HashMap;

use crate::system::System;
use crate::texture_holder::TextureHolder;
use crate::ONE_SECOND;

const DEATH_SPRITE_WIDTH: u32 = 30;
const DEATH_SPRITE_HEIGHT: u32 = 22;
const LEVEL_UP_SPRITE_WIDTH: u32 = 82;
const LEVEL_UP_SPRITE_HEIGHT: u32 = 36;

pub fn create_death_animation_texture<'a>(
    textures: &mut HashMap<String, TextureHolder<'a>>,
    texture_creator: &'a TextureCreator<WindowContext>,
) {
    textures.insert(
        "death".to_owned(),
        TextureHolder::from_image(texture_creator, "resources/death.png"),
    );
}

pub fn create_level_up_animation_texture<'a>(
    textures: &mut HashMap<String, TextureHolder<'a>>,
    texture_creator: &'a TextureCreator<WindowContext>,
) {
    textures.insert(
        "level-up".to_owned(),
        TextureHolder::from_image(texture_creator, "resources/level-up.png"),
    );
}

pub struct Animation<'a> {
    pub texture: &'a TextureHolder<'a>,
    nb_animations: u32,
    duration: u64,
    max_duration: u64,
    sprite_width: u32,
    sprite_height: u32,
    pub sprite_display_width: u32,
    pub sprite_display_height: u32,
}

impl<'a> Animation<'a> {
    pub fn new_death(textures: &'a HashMap<String, TextureHolder<'a>>) -> Animation<'a> {
        let texture = &textures[&"death".to_owned()];
        let nb_animations = texture.width / DEATH_SPRITE_WIDTH;
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

    pub fn new_level_up(textures: &'a HashMap<String, TextureHolder<'a>>) -> Animation<'a> {
        let texture = &textures[&"level-up".to_owned()];
        let nb_animations = texture.width / LEVEL_UP_SPRITE_WIDTH;
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

    pub fn update(&mut self, elapsed: u64) {
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
        system
            .canvas
            .copy(
                &self.texture.texture,
                Rect::new(tile_x as i32, 0, self.sprite_width, self.sprite_height),
                Rect::new(x, y, self.sprite_display_width, self.sprite_display_height),
            )
            .expect("copy animation failed");
    }

    pub fn is_done(&self) -> bool {
        self.duration >= self.max_duration
    }
}
