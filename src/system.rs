use crate::sdl2::pixels::Color;
use crate::sdl2::render::{Canvas, TextureCreator};
use crate::sdl2::ttf::Font;
use crate::sdl2::video::{Window, WindowContext};

use crate::font_handler::FontHandler;
use crate::health_bar::HealthBar;
use crate::player::Player;

pub struct System<'a> {
    pub canvas: Canvas<Window>,
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
    pub health_bar: &'a HealthBar<'a>,
    pub font_maps: Vec<FontHandler<'a>>,
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
            font_maps: Vec::new(),
        }
    }

    pub fn create_new_font_map<'b>(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'b Font<'b, 'static>,
        font_size: u16,
        color: Color,
    ) {
        if self
            .font_maps
            .iter()
            .any(|f| f.color == color && f.size == font_size)
        {
            return;
        }
        self.font_maps
            .push(FontHandler::new(texture_creator, font, font_size, color));
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
        self.canvas.window().gl_swap_window();
        self.canvas.clear();
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font_size: u16,
        color: Color,
        x: i32,
        y: i32,
        x_centered: bool,
        y_centered: bool,
    ) -> (u32, u32) {
        if let Some(pos) = self
            .font_maps
            .iter()
            .position(|f| f.color == color && f.size == font_size)
        {
            // Very ugly hack to be able to send &mut self while borrowing `self.font_maps`!
            let font = &self.font_maps[pos] as *const FontHandler;
            unsafe { (*font).draw(self, text, x, y, x_centered, y_centered) }
        } else {
            (0, 0)
        }
    }

    /// The purpose is just to display the font map.
    pub fn full_draw_text(&mut self, x: i32, y: i32) {
        use crate::sdl2::rect::Rect;
        self.canvas
            .copy(
                &self.font_maps[1].texture.texture,
                None,
                Rect::new(
                    x,
                    y,
                    self.font_maps[1].texture.width,
                    self.font_maps[1].texture.height,
                ),
            )
            .unwrap();
    }
}
