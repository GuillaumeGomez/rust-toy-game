use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::character::Direction;
use crate::GetDimension;

pub trait Weapon {
    fn draw(&mut self, x: i32, y: i32, canvas: &mut Canvas<Window>, screen: &Rect);
    /// Returns `false` if there is already an action in progress.
    fn use_it(&mut self, direction: Direction) -> bool;
    fn stop_use(&mut self);
    fn weight(&self) -> u32;
}

#[allow(dead_code)]
pub enum WeaponHandler<'a> {
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

impl<'a> Weapon for WeaponHandler<'a> {
    fn draw(&mut self, x: i32, y: i32, canvas: &mut Canvas<Window>, screen: &Rect) {
        match *self {
            Self::Sword(ref mut s) => s.draw(x, y, canvas, screen),
            _ => {}
        }
    }
    fn use_it(&mut self, direction: Direction) -> bool {
        match *self {
            Self::Sword(ref mut s) => s.use_it(direction),
            _ => false,
        }
    }
    fn stop_use(&mut self) {
        match *self {
            Self::Sword(ref mut s) => s.stop_use(),
            _ => {}
        }
    }
    fn weight(&self) -> u32 {
        0
    }
}

impl<'a> GetDimension for WeaponHandler<'a> {
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
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> WeaponHandler<'a> {
        let surface = Surface::from_file("resources/weapon.png")
            .expect("failed to load `resources/weapon.png`");

        let width = surface.width();
        let height = surface.height();
        WeaponHandler::Sword(Sword {
            texture: texture_creator
                .create_texture_from_surface(surface)
                .expect("failed to build weapon texture from surface"),
            action: None,
            width,
            height,
        })
    }
}

impl<'a> Weapon for Sword<'a> {
    fn draw(&mut self, x: i32, y: i32, canvas: &mut Canvas<Window>, screen: &Rect) {
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
                action.angle += 5;
                self.action = Some(action);
            }
        }
    }
    fn use_it(&mut self, direction: Direction) -> bool {
        if self.action.is_some() {
            return false;
        }
        let (angle, max_angle, x_add, y_add) = match direction {
            Direction::Up => (-45, 45, self.width() as i32 / 2, self.height() as i32),
            Direction::Down => (135, 225, self.width() as i32 / 2, 0),
            Direction::Left => (225, 315, self.width() as i32, self.height() as i32 / 2),
            Direction::Right => (45, 135, 0, self.height() as i32 / 2),
        };
        self.action = Some(WeaponAction { angle, max_angle, x_add, y_add });
        true
    }
    fn stop_use(&mut self) {
        self.action = None;
    }
    fn weight(&self) -> u32 {
        // TODO: when inventory will be a thing
        0
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
