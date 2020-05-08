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
    action: Option<WeaponAction>,
    pub kind: WeaponKind<'a>,
}

impl<'a> Weapon<'a> {
    /// Returns (base_point, new_head_point)
    pub fn compute_angle(&self) -> Option<((i32, i32), (i32, i32))> {
        let action = self.get_action()?;
        let height = self.height() as i32;
        let width = self.width() as i32 / 2;

        let radian_angle = action.angle as f32 * 0.0174533;
        let x2 = 0. - height as f32 * radian_angle.sin() + width as f32;
        let y2 = 0. + height as f32 * radian_angle.cos();
        Some((
            (self.x + action.x_add, self.y + action.y_add),
            (
                self.x - x2 as i32 + action.x_add,
                self.y + action.y_add - y2 as i32,
            ),
        ))
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        if let Some(mut action) = self.action.take() {
            if let Some(texture) = self.get_texture() {
                let x = self.x - screen.x;
                let y = self.y - screen.y;
                canvas
                    .copy_ex(
                        texture,
                        None,
                        Rect::new(x, y, self.width(), self.height()),
                        action.angle as _,
                        Some((action.x_add, action.y_add).into()),
                        false,
                        false,
                    )
                    .expect("failed to copy sword");
                // let ((x1, y1), (x2, y2)) = return_if_none!(self.compute_angle());
                // let x = if x1 < x2 { x1 } else { x2 };
                // let y = if y1 < y2 { y1 } else { y2 };
                // let width = (x1 - x2).abs() as u32;
                // let height = (y1 - y2).abs() as u32;

                // canvas.fill_rect(Rect::new(x - screen.x, y - screen.y, width, height));
            }
            if action.angle < action.max_angle {
                action.angle += 10;
                self.action = Some(action);
            }
        }
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
    fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        match *self {
            Self::Sword(ref mut s) => s.use_it(direction),
            _ => None,
        }
    }
    pub fn weight(&self) -> u32 {
        0
    }
    pub fn get_texture(&self) -> Option<&Texture<'a>> {
        match *self {
            Self::Sword(ref s) => Some(s.get_texture()),
            _ => None,
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
            action: None,
            kind: WeaponKind::Sword(Sword {
                texture: texture_creator
                    .create_texture_from_surface(surface)
                    .expect("failed to build weapon texture from surface"),
                width,
                height,
            }),
        }
    }
    /// In case there is a timeout or something, you might not be able to use the weapon.
    pub fn use_it(&mut self, direction: Direction) -> Option<WeaponAction> {
        let (angle, max_angle, x_add, y_add) = match direction {
            Direction::Up => (-45, 45, self.width() as i32 / 2, self.height() as i32),
            Direction::Down => (135, 225, self.width() as i32 / 2, self.height() as i32),
            Direction::Left => (225, 315, 0, self.height() as i32),
            Direction::Right => (45, 135, 0, self.height() as i32),
        };
        Some(WeaponAction {
            angle,
            max_angle,
            x_add,
            y_add,
        })
    }
    pub fn weight(&self) -> u32 {
        // TODO: when inventory will be a thing
        0
    }
    pub fn get_texture(&self) -> &Texture<'a> {
        &self.texture
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
