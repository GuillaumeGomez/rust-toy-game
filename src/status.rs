use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

pub struct Status<'a> {
    texture: Texture<'a>,
    width: i32,
    height: i32,
    // When it reaches y_limit, the status should be removed.
    y_pos: i32,
    y_limit: i32,
}

impl<'a> Status<'a> {
    pub fn new<'b>(
        font: &'b Font<'b, 'static>,
        texture_creator: &'a TextureCreator<WindowContext>,
        font_size: i32,
        text: &str,
        color: Color,
    ) -> Status<'a> {
        let text_surface = font
            .render(text)
            .solid(color)
            .expect("failed to convert text to surface");
        let width = text_surface.width() as i32;
        let height = text_surface.height() as i32;
        let text_texture = texture_creator
            .create_texture_from_surface(text_surface)
            .expect("failed to build texture from debug surface");
        Status {
            texture: text_texture,
            width,
            height,
            y_pos: 0,
            y_limit: 30,
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect, x: i32, y: i32) {
        self.y_pos += 1; // increase position of the text
        let x = x - screen.x - self.width / 2;
        let y = y - screen.y - self.y_pos - 10;
        if x + self.width >= 0
            && x < screen.width() as i32
            && y + self.height >= 0
            && y < screen.height() as i32
        {
            canvas
                .copy(
                    &self.texture,
                    None,
                    Rect::new(x, y, self.width as u32, self.height as u32),
                )
                .expect("copy status failed");
        }
    }

    pub fn should_be_removed(&self) -> bool {
        self.y_pos >= self.y_limit
    }
}
