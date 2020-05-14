use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::health_bar::HealthBar;
use crate::player::Player;

pub struct System<'a> {
    pub canvas: Canvas<Window>,
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
    pub health_bar: &'a HealthBar<'a>,
}

impl<'a> System<'a> {
    pub fn new(
        canvas: Canvas<Window>,
        width: u32,
        height: u32,
        health_bar: &'a HealthBar,
    ) -> System<'a> {
        System {
            canvas,
            x: 0,
            y: 0,
            width,
            height,
            health_bar,
        }
    }

    pub fn set_screen_position(&mut self, player: &Player) {
        self.x = player.x - self.width() as i64 / 2;
        self.y = player.y - self.height() as i64 / 2;
    }

    pub fn width(&self) -> i32 {
        self.width as i32
    }

    pub fn height(&self) -> i32 {
        self.height as i32
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn clear(&mut self) {
        self.canvas.present();
        self.canvas.clear();
    }
}
