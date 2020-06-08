use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::TextureHolder;
use crate::GetDimension;

struct TitleBarButton<'a> {
    texture: Texture<'a>,
    texture_pressed: Texture<'a>,
    texture_hovered: Texture<'a>,
    texture_hovered_and_pressed: Texture<'a>,
    size: u32,
    is_hovered: bool,
    is_pressed: bool,
}

impl<'a> TitleBarButton<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>, size: u32) -> TitleBarButton<'a> {
        let create_button = |button_color: Color, cross_color: Color| {
            let mut button = Surface::new(size, size, texture_creator.default_pixel_format())
                .expect("Failed to create surface for titlebar button");
            button
                .fill_rect(None, button_color)
                .expect("failed to create window titlebar button");
            let mut canvas =
                Canvas::from_surface(button).expect("failed to create Canvas from surface");
            canvas.set_draw_color(cross_color);
            canvas
                .draw_line((3, 3), (size as i32 - 4, size as i32 - 4))
                .expect("failed to draw line");
            canvas
                .draw_line((size as i32 - 4, 3), (3, size as i32 - 4))
                .expect("failed to draw line");
            texture_creator
                .create_texture_from_surface(canvas.into_surface())
                .expect("failed to build texture from surface")
        };

        TitleBarButton {
            texture: create_button(Color::RGB(255, 0, 0), Color::RGB(255, 255, 255)),
            texture_pressed: create_button(Color::RGB(220, 0, 0), Color::RGB(255, 255, 255)),
            texture_hovered: create_button(Color::RGB(255, 0, 0), Color::RGB(200, 200, 255)),
            texture_hovered_and_pressed: create_button(
                Color::RGB(220, 0, 0),
                Color::RGB(200, 200, 255),
            ),
            size,
            is_hovered: false,
            is_pressed: false,
        }
    }

    fn draw(&self, system: &mut System, x: i32, y: i32) {
        let t = if self.is_pressed && self.is_hovered {
            &self.texture_hovered_and_pressed
        } else if self.is_pressed {
            &self.texture_pressed
        } else if self.is_hovered {
            &self.texture_hovered
        } else {
            &self.texture
        };
        system
            .canvas
            .copy(t, None, Rect::new(x, y, self.size, self.size))
            .expect("failed to draw titlebar button");
    }
}

pub struct Window<'a> {
    title_bar_height: u32,
    border_width: u32,
    texture: TextureHolder<'a>,
    button: TitleBarButton<'a>,
    x: i32,
    y: i32,
    is_hidden: bool,
    is_dragging_window: Option<(i32, i32)>,
}

impl<'a> Window<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Window<'a> {
        let title_bar_height = 20;
        let border_width = 1;
        let mut window = Surface::new(width, height, texture_creator.default_pixel_format())
            .expect("Failed to create surface for font map");
        window
            .fill_rect(None, Color::RGB(110, 110, 110))
            .expect("failed to create window background");
        window
            .fill_rect(
                Rect::new(
                    border_width as i32,
                    title_bar_height as i32,
                    width - border_width * 2,
                    height - border_width - title_bar_height,
                ),
                Color::RGB(239, 239, 239),
            )
            .expect("failed to create title bar");
        Window {
            title_bar_height,
            border_width,
            texture: TextureHolder::surface_to_texture(texture_creator, window),
            x,
            y,
            is_hidden: true,
            is_dragging_window: None,
            button: TitleBarButton::new(texture_creator, title_bar_height - 6),
        }
    }

    pub fn show(&mut self) {
        self.is_hidden = false;
    }

    pub fn hide(&mut self) {
        self.is_hidden = true;
        self.is_dragging_window = None;
    }

    pub fn draw(&self, system: &mut System) {
        if self.is_hidden() {
            return;
        }
        system
            .canvas
            .copy(
                &self.texture.texture,
                None,
                Rect::new(self.x, self.y, self.texture.width, self.texture.height),
            )
            .expect("failed to draw window");
        self.button.draw(
            system,
            // 3 is half the border around the titlebar button
            self.x + self.texture.width as i32 - self.title_bar_height as i32 + 3,
            self.y + 3,
        );
    }

    pub fn is_in_titlebar_button(&self, mouse_x: i32, mouse_y: i32) -> bool {
        let x = self.x + self.texture.width as i32 - self.title_bar_height as i32 + 3;
        let y = self.y + 3;
        !(mouse_x < x
            || mouse_x > x + self.button.size as i32
            || mouse_y < y
            || mouse_y > y + self.button.size as i32)
    }

    pub fn handle_event(&mut self, ev: &Event) {
        if self.is_hidden || (!ev.is_mouse() && !ev.is_controller()) {
            return;
        }
        match ev {
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                ref x,
                ref y,
                ..
            } => {
                if *x >= self.x
                    && *x <= self.x + self.width() as i32
                    && *y >= self.y
                    && *y <= self.y + self.title_bar_height as i32
                {
                    if self.is_in_titlebar_button(*x, *y) {
                        self.button.is_pressed = true;
                    } else {
                        self.is_dragging_window = Some((*x - self.x, *y - self.y));
                    }
                }
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.is_dragging_window = None;
                if self.button.is_pressed && self.is_in_titlebar_button(*x, *y) {
                    self.is_hidden = true;
                    self.button.is_hovered = true;
                }
                self.button.is_pressed = false;
            }
            Event::MouseMotion {
                x: mouse_x,
                y: mouse_y,
                ..
            } => match self.is_dragging_window {
                Some((x_add, y_add)) => {
                    self.x = *mouse_x - x_add;
                    self.y = *mouse_y - y_add;
                }
                None => {
                    self.button.is_hovered = self.is_in_titlebar_button(*mouse_x, *mouse_y);
                }
            },
            _ => {}
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.is_hidden
    }
}

impl<'a> GetDimension for Window<'a> {
    fn width(&self) -> u32 {
        self.texture.width
    }
    fn height(&self) -> u32 {
        self.texture.height
    }
}
