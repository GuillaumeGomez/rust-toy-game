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
use crate::{GetDimension, GetPos, ONE_SECOND};

#[allow(dead_code)]
pub enum WeaponKind {
    Sword(Sword),
    LongSword,
    Axe,
    Arc,
    Mass,
    Hammer,
    Spear,
    Dagger,
    Wand,
}

pub struct Weapon {
    pub x: i64,
    pub y: i64,
    action: Option<WeaponAction>,
    blocking_direction: Option<Direction>,
    pub kind: WeaponKind,
    data_id: &'static str,
    /// Total time required for this weapon to perform its action.
    pub total_time: u64,
    pub attack: i32,
}

impl Weapon {
    /// Returns a vec of positions to check.
    // FIXME: This whole thing is terrible performance-wise...
    pub fn compute_angle(&self, textures: &Textures<'_>) -> Option<Vec<(i64, i64)>> {
        let action = self.get_action()?;
        let height = self.height() as i64;
        let width = self.width() as i64;
        let half_width = width / 2;
        let data = textures.get_data(self.data_id);
        let mut matrix = Vec::with_capacity(data.len());

        let radian_angle = action.angle as f32 * 0.0174533;
        let radian_sin = radian_angle.sin();
        let radian_cos = radian_angle.cos();
        for y in 0..height {
            let y = (height - 1 - y) as usize;
            let y_f32 = y as f32;
            let rad_sin_y = y_f32 * radian_sin;
            let rad_cos_y = y_f32 * radian_cos;
            for x in 0..width {
                if unsafe { *data.get_unchecked(y * width as usize + x as usize) } == 0 {
                    continue;
                }
                let x = x - half_width;
                let x2 = x as f32 * radian_cos - rad_sin_y;
                let y2 = x as f32 * radian_sin + rad_cos_y;
                matrix.push((self.x - x2 as i64, self.y - y2 as i64 + action.y_add as i64));
            }
        }
        Some(matrix)
    }
    /// Set the position based on the character and its direction.
    pub fn set_pos(&mut self, x: i64, y: i64) {
        self.x = x;
        self.y = y;
    }
    pub fn update(&mut self, elapsed: u64) {
        if self.blocking_direction.is_some() {
            return;
        }
        if let Some(mut action) = self.action.take() {
            if action.duration > elapsed {
                action.duration -= elapsed;

                let angle_add = elapsed * action.total_angle as u64 / self.total_time as u64;

                action.angle += angle_add as i32;
                self.action = Some(action);
            }
        }
    }
    pub fn draw(&self, system: &mut System, _debug: bool) {
        if let Some(direction) = self.blocking_direction {
            if let Some(texture) = self.get_texture() {
                let x = self.x - system.x();
                let y = self.y - system.y();
                let (angle, _, _) = match direction {
                    Direction::Up => (90, 0, self.height() as i32),
                    Direction::Right => (180, self.width() as i32, 0),
                    Direction::Down => (270, self.width() as i32, self.height() as i32),
                    Direction::Left => (0, 0, 0),
                };
                system.copy_ex_to_canvas(
                    texture,
                    None,
                    Rect::new(x as i32, y as i32, self.width(), self.height()),
                    angle as f64,
                    None,
                    false,
                    false,
                );
            }
        } else if let Some(ref action) = self.action {
            if let Some(texture) = self.get_texture() {
                let x = self.x - system.x();
                let y = self.y - system.y();
                system.copy_ex_to_canvas(
                    texture,
                    None,
                    Rect::new(x as i32, y as i32, self.width(), self.height()),
                    action.angle as f64,
                    Some((action.x_add, action.y_add).into()),
                    false,
                    false,
                );
            }
        }
    }
    pub fn is_blocking(&self) -> bool {
        self.blocking_direction.is_some()
    }
    pub fn block(&mut self, direction: Direction) {
        self.blocking_direction = Some(direction);
        self.action = None;
    }
    pub fn stop_block(&mut self) {
        self.blocking_direction = None;
    }
    pub fn is_attacking(&self) -> bool {
        self.action.is_some()
    }
    pub fn get_action(&self) -> Option<&WeaponAction> {
        self.action.as_ref()
    }
    pub fn stop_use(&mut self) {
        self.action = None;
    }
    pub fn use_it(&mut self, direction: Direction) {
        self.action = self.kind.use_it(direction);
        self.blocking_direction = None;
    }
}

impl GetPos for Weapon {
    fn x(&self) -> i64 {
        self.x
    }
    fn y(&self) -> i64 {
        self.y
    }
}

impl GetDimension for Weapon {
    fn width(&self) -> u32 {
        self.kind.width()
    }
    fn height(&self) -> u32 {
        self.kind.height()
    }
}

impl Deref for Weapon {
    type Target = WeaponKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl DerefMut for Weapon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl WeaponKind {
    fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        match *self {
            Self::Sword(ref mut s) => s.use_it(direction),
            _ => None,
        }
    }
    pub fn weight(&self) -> u32 {
        match *self {
            Self::Sword(ref s) => s.weight(),
            _ => 0,
        }
    }
    pub fn get_texture(&self) -> Option<TextureId> {
        match *self {
            Self::Sword(ref s) => Some(s.get_texture()),
            _ => None,
        }
    }
}

impl GetDimension for WeaponKind {
    fn width(&self) -> u32 {
        match *self {
            Self::Sword(ref s) => s.width(),
            _ => 0,
        }
    }
    fn height(&self) -> u32 {
        match *self {
            Self::Sword(ref s) => s.height(),
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct WeaponAction {
    angle: i32,
    /// The angle of the rotation.
    total_angle: u32,
    duration: u64,
    x_add: i32,
    y_add: i32,
}

pub struct Sword {
    texture: TextureId,
}

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
            action: None,
            data_id: "sword",
            total_time: ONE_SECOND / 4,
            kind: WeaponKind::Sword(Sword {
                texture: textures.get_texture_id_from_name("sword"),
            }),
            attack,
            blocking_direction: None,
        }
    }
    /// In case there is a timeout or something, you might not be able to use the weapon.
    pub fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        let (angle, x_add, y_add) = match direction {
            Direction::Up => (-45, self.width() as i32 / 2, self.height() as i32),
            Direction::Down => (135, self.width() as i32 / 2, self.height() as i32),
            Direction::Left => (225, 0, self.height() as i32),
            Direction::Right => (45, 0, self.height() as i32),
        };
        Some(WeaponAction {
            angle,
            total_angle: 90,
            duration: ONE_SECOND / 4,
            x_add,
            y_add,
        })
    }
    pub fn weight(&self) -> u32 {
        10
    }
    pub fn get_texture(&self) -> TextureId {
        self.texture
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
