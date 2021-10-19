use std::ops::{Deref, DerefMut};

use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::PixelFormatEnum;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::{Texture, TextureCreator};
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::character::Direction;
use crate::system::System;
use crate::texture_holder::{TextureHolder, TextureId, Textures};
use crate::weapon::{Weapon, WeaponAction, WeaponActionKind, WeaponKind};
use crate::{GetDimension, GetPos, ONE_SECOND};

fn get_surface_data(surface: &Surface<'_>) -> Vec<u8> {
    let height = surface.height() as usize;
    let width = surface.width() as usize;
    let mut data = Vec::with_capacity(width * height);

    let surface = surface.raw();
    let pixels = unsafe { (*surface).pixels as *const u32 };

    for pos in 0..height * width {
        let target_pixel = unsafe { *(pixels.add(pos) as *const u32) };
        let alpha = target_pixel & 255;
        data.push(if alpha > 220 { 1 } else { 0 });
    }
    data
}

pub struct Sword {
    texture: TextureId,
}

impl Sword {
    pub fn init_textures<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
    ) {
        let mut surface = Surface::from_file("resources/weapon.png")
            .expect("failed to load `resources/weapon.png`");

        if surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
            surface = surface
                .convert_format(PixelFormatEnum::RGBA8888)
                .expect("failed to convert surface to RGBA8888");
        }

        let data = get_surface_data(&surface);
        textures.add_surface_data("sword", data);
        textures.add_named_texture(
            "sword",
            TextureHolder::surface_into_texture(texture_creator, surface),
        );
    }

    pub fn new(textures: &Textures<'_>, attack: i32) -> Weapon {
        Weapon {
            x: 0,
            y: 0,
            data_id: "sword",
            total_time: ONE_SECOND as u32 / 4,
            kind: WeaponKind::Sword(Sword {
                texture: textures.get_texture_id_from_name("sword"),
            }),
            attack,
        }
    }
    /// In case there is a timeout or something, you might not be able to use the weapon.
    pub fn use_it(&mut self, direction: Direction, total_duration: u32) -> Option<WeaponAction> {
        let (start_angle, x_add, y_add) = match direction {
            Direction::Up => (-45, self.width() as i32 / 2, self.height() as i32),
            Direction::Down => (135, self.width() as i32 / 2, self.height() as i32),
            Direction::Left => (225, 0, self.height() as i32),
            Direction::Right => (45, 0, self.height() as i32),
        };
        Some(WeaponAction {
            duration: 0,
            total_duration,
            x_add,
            y_add,
            kind: WeaponActionKind::AttackBySlash {
                start_angle,
                total_angle: 90,
            },
        })
    }
    pub fn weight(&self) -> u32 {
        10
    }
    pub fn get_texture(&self) -> Option<TextureId> {
        Some(self.texture)
    }
}

impl GetDimension for Sword {
    fn width(&self) -> u32 {
        self.texture.width as _
    }
    fn height(&self) -> u32 {
        self.texture.height as _
    }
}
