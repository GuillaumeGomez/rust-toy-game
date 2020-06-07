use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::TextureHolder;
use crate::GetDimension;

pub struct Window<'a> {
    title_bar_height: u32,
    border_width: u32,
    texture: TextureHolder<'a>,
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
                    self.is_dragging_window = Some((*x - self.x, *y - self.y));
                }
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.is_dragging_window = None;
            }
            Event::MouseMotion { ref x, ref y, .. } => match self.is_dragging_window {
                Some((x_add, y_add)) => {
                    self.x = *x - x_add;
                    self.y = *y - y_add;
                }
                _ => {}
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
