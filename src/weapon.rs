use std::ops::{Deref, DerefMut};

use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::character::Direction;
use crate::GetDimension;

#[allow(dead_code)]
pub enum WeaponKind<'a> {
    Sword(Sword<'a>),
    LongSword,
    Axe,
    Arc,
    Mass,
    Hammer,
    Spear,
    Dagger,
    Wand,
}

pub struct Weapon<'a> {
    pub x: i32,
    pub y: i32,
    pub kind: WeaponKind<'a>,
}

impl<'a> Weapon<'a> {
    pub fn compute_angle(&self) -> Option<(i32, i32)> {
        let angle = match self.kind {
            WeaponKind::Sword(ref s) => s.get_angle(),
            _ => None,
        }?;
        let x1 = self.kind.width() / 2;
        let y1 = 0 - self.kind.height() as i32;

        let radian_angle = angle as f32 * 0.0174533;
        let x2 = x1 as f32 * radian_angle.cos() - y1 as f32 * radian_angle.sin();
        let y2 = x1 as f32 * radian_angle.sin() + y1 as f32 * radian_angle.cos();
        Some((x2 as i32 + self.x, y2 as i32 + self.y))
    }
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        match self.kind {
            WeaponKind::Sword(ref mut s) => s.draw(self.x, self.y, canvas, screen),
            _ => {}
        }
    }
}

impl<'a> Deref for Weapon<'a> {
    type Target = WeaponKind<'a>;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl<'a> DerefMut for Weapon<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl<'a> WeaponKind<'a> {
    pub fn use_it(&mut self, direction: Direction) {
        match *self {
            Self::Sword(ref mut s) => s.use_it(direction),
            _ => {}
        }
    }
    pub fn stop_use(&mut self) {
        match *self {
            Self::Sword(ref mut s) => s.stop_use(),
            _ => {}
        }
    }
    pub fn weight(&self) -> u32 {
        0
    }
    pub fn is_attacking(&self) -> bool {
        match *self {
            Self::Sword(ref s) => s.is_attacking(),
            _ => false,
        }
    }
}

impl<'a> GetDimension for WeaponKind<'a> {
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
struct WeaponAction {
    angle: i32,
    max_angle: i32,
    x_add: i32,
    y_add: i32,
}

pub struct Sword<'a> {
    texture: Texture<'a>,
    action: Option<WeaponAction>,
    width: u32,
    height: u32,
}

impl<'a> Sword<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Weapon<'a> {
        let surface = Surface::from_file("resources/weapon.png")
            .expect("failed to load `resources/weapon.png`");

        let width = surface.width();
        let height = surface.height();
        Weapon {
            x: 0,
            y: 0,
            kind: WeaponKind::Sword(Sword {
                texture: texture_creator
                    .create_texture_from_surface(surface)
                    .expect("failed to build weapon texture from surface"),
                action: None,
                width,
                height,
            }),
        }
    }
    pub fn draw(&mut self, x: i32, y: i32, canvas: &mut Canvas<Window>, screen: &Rect) {
        if let Some(mut action) = self.action.take() {
            let x = x - screen.x;
            let y = y - screen.y;
            canvas
                .copy_ex(
                    &self.texture,
                    None,
                    Rect::new(x, y, self.width, self.height),
                    action.angle as _,
                    Some((action.x_add, action.y_add).into()),
                    false,
                    false,
                )
                .expect("failed to copy sword");
            if action.angle < action.max_angle {
                action.angle += 10;
                self.action = Some(action);
            }
        }
    }
    pub fn use_it(&mut self, direction: Direction) {
        let (angle, max_angle, x_add, y_add) = match direction {
            Direction::Up => (-45, 45, self.width() as i32 / 2, self.height() as i32),
            Direction::Down => (135, 225, self.width() as i32 / 2, self.height() as i32),
            Direction::Left => (225, 315, 0, self.height() as i32),
            Direction::Right => (45, 135, 0, self.height() as i32),
        };
        self.action = Some(WeaponAction {
            angle,
            max_angle,
            x_add,
            y_add,
        });
    }
    pub fn stop_use(&mut self) {
        self.action = None;
    }
    pub fn weight(&self) -> u32 {
        // TODO: when inventory will be a thing
        0
    }
    pub fn is_attacking(&self) -> bool {
        self.action.is_some()
    }
    fn get_angle(&self) -> Option<i32> {
        self.action.as_ref().map(|action| action.angle)
    }
}

impl<'a> GetDimension for Sword<'a> {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}
