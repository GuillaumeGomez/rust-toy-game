use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::player::Player;

pub struct System {
    pub canvas: Canvas<Window>,
    pub screen: Rect,
}

impl System {
    pub fn new(canvas: Canvas<Window>, width: u32, height: u32) -> System {
        System {
            canvas,
            screen: Rect::new(0, 0, width, height),
        }
    }

    pub fn set_screen_position(&mut self, player: &Player) {
        self.screen.x = player.x - self.width() / 2;
        self.screen.y = player.y - self.height() / 2;
    }

    pub fn width(&self) -> i32 {
        self.screen.width() as i32
    }

    pub fn height(&self) -> i32 {
        self.screen.height() as i32
    }

    pub fn x(&self) -> i32 {
        self.screen.x
    }

    pub fn y(&self) -> i32 {
        self.screen.y
    }

    pub fn clear(&mut self) {
        self.canvas.present();
        self.canvas.clear();
    }
}
