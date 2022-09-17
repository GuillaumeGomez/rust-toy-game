use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::PixelFormatEnum;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::{TextureId, Textures};
use crate::{Draw, GetDimension, GetPos, TextureHolder};

pub enum BuildingKind {
    WeaponShop,
}

impl BuildingKind {
    // (width, height, x, y)
    fn sprite_dimensions(&self) -> (u32, u32, f32, f32) {
        match *self {
            Self::WeaponShop => (108, 110, 322., 32.),
        }
    }
}

pub struct Building {
    pub x: f32,
    pub y: f32,
    pub kind: BuildingKind,
    texture: TextureId,
}

impl Building {
    pub fn init_textures<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
    ) {
        let mut surface = Surface::from_file("resources/building.png")
            .expect("failed to load `resources/building.png`");

        if surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
            surface = surface
                .convert_format(PixelFormatEnum::RGBA8888)
                .expect("failed to convert surface to RGBA8888");
        }
        textures.add_named_texture(
            "building",
            TextureHolder::surface_to_texture(texture_creator, &surface),
        );
        // textures.add_surface("building", surface);
    }

    pub fn new(x: f32, y: f32, kind: BuildingKind, textures: &Textures<'_>) -> Self {
        Self { x, y, kind, texture: textures.get_texture_id_from_name("building") }
    }
}

impl Draw for Building {
    fn draw(&mut self, system: &mut System, _debug: bool) {
        let (width, height, sprite_x, sprite_y) = self.kind.sprite_dimensions();

        let x = (self.x - system.x()) as i32 - (width / 2) as i32;
        let y = (self.y - system.y()) as i32 - (height / 2) as i32;

        if width as i32 + x < 0
            || x > system.width() as i32
            || height as i32 + y < 0
            || y > system.height() as i32
        {
            return;
        }
        system.copy_to_canvas(
            self.texture,
            Rect::new(sprite_x as _, 0, width, height),
            Rect::new(x, y, width, height),
        );
    }
}

impl GetPos for Building {
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
}
