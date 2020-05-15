use sdl2::event::Event;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::system::System;

struct Button<'a> {
    texture: Texture<'a>,
    texture_clicked: Texture<'a>,
    text: Texture<'a>,
    text_hovered: Texture<'a>,
    rect: Rect,
    text_width: u32,
    text_height: u32,
    is_hovered: bool,
    is_clicked: bool,
}

impl<'a> Button<'a> {
    fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'a Font,
        text: &str,
        rect: Rect,
    ) -> Button<'a> {
        let mut button = Surface::new(
            rect.width(),
            rect.height(),
            texture_creator.default_pixel_format(),
        )
        .expect("failed to create button surface");
        button
            .fill_rect(None, Color::RGB(30, 30, 30))
            .expect("failed to fill button surface");
        let mut button_clicked = Surface::new(
            rect.width(),
            rect.height(),
            texture_creator.default_pixel_format(),
        )
        .expect("failed to create button surface");
        button_clicked
            .fill_rect(None, Color::RGB(20, 20, 20))
            .expect("failed to fill button surface");
        let text_surface = font
            .render(text)
            .blended(Color::RGB(255, 255, 255))
            .expect("failed to convert text to surface");
        let text_width = text_surface.width();
        let text_height = text_surface.height();
        let text_hover_surface = font
            .render(text)
            .blended(Color::RGB(74, 138, 221))
            .expect("failed to convert text to surface");

        Button {
            texture: texture_creator
                .create_texture_from_surface(button)
                .expect("failed to build texture from Button surface"),
            texture_clicked: texture_creator
                .create_texture_from_surface(button_clicked)
                .expect("failed to build texture from Button surface"),
            text: texture_creator
                .create_texture_from_surface(text_surface)
                .expect("failed to build texture from Button surface"),
            text_hovered: texture_creator
                .create_texture_from_surface(text_hover_surface)
                .expect("failed to build texture from Button surface"),
            rect,
            text_width,
            text_height,
            is_hovered: false,
            is_clicked: false,
        }
    }

    fn is_in(&self, mouse_x: i32, mouse_y: i32) -> bool {
        !(mouse_x < self.rect.x
            || mouse_x > self.rect.x + self.rect.width() as i32
            || mouse_y < self.rect.y
            || mouse_y > self.rect.y + self.rect.height() as i32)
    }

    fn update(&mut self, mouse_x: i32, mouse_y: i32) {
        self.is_hovered = self.is_in(mouse_x, mouse_y);
    }

    fn update_click(&mut self, mouse_x: i32, mouse_y: i32) {
        self.is_clicked = self.is_in(mouse_x, mouse_y);
    }

    fn draw(&self, system: &mut System) {
        system
            .canvas
            .copy(
                if self.is_clicked {
                    &self.texture_clicked
                } else {
                    &self.texture
                },
                None,
                self.rect,
            )
            .expect("copy menu failed");
        system
            .canvas
            .copy(
                if self.is_hovered {
                    &self.text_hovered
                } else {
                    &self.text
                },
                None,
                Rect::new(
                    self.rect.x + (self.rect.width() - self.text_width) as i32 / 2,
                    self.rect.y + (self.rect.height() - self.text_height) as i32 / 2,
                    self.text_width,
                    self.text_height,
                ),
            )
            .expect("copy menu failed");
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MenuEvent {
    Quit,
    Resume,
    None,
}

pub struct Menu<'a> {
    background: Texture<'a>,
    button_resume: Button<'a>,
    button_quit: Button<'a>,
    selected: Option<usize>,
    selected_texture: Texture<'a>,
}

impl<'a> Menu<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'a Font,
        width: u32,
        height: u32,
    ) -> Menu<'a> {
        let mut background = Surface::new(width, height, PixelFormatEnum::RGBA8888)
            .expect("failed to create background surface");
        background
            .fill_rect(None, Color::RGBA(0, 0, 0, 170))
            .expect("failed to fill background surface");
        let mut selected_surface = Surface::new(20, 20, texture_creator.default_pixel_format())
            .expect("failed to create selected surface");
        selected_surface
            .fill_rect(None, Color::RGB(74, 138, 221))
            .expect("failed to fill selected surface");

        Menu {
            background: texture_creator
                .create_texture_from_surface(background)
                .expect("failed to build texture from Menu surface"),
            button_resume: Button::new(
                texture_creator,
                font,
                "Resume",
                Rect::new(width as i32 / 4, height as i32 / 3, width / 2, 50),
            ),
            button_quit: Button::new(
                texture_creator,
                font,
                "Quit",
                Rect::new(width as i32 / 4, height as i32 / 3 * 2, width / 2, 50),
            ),
            selected_texture: texture_creator
                .create_texture_from_surface(selected_surface)
                .expect("failed to build texture from selected surface"),
            selected: None,
        }
    }

    pub fn update(&mut self, mouse_x: i32, mouse_y: i32) {
        self.button_resume.update(mouse_x, mouse_y);
        self.button_quit.update(mouse_x, mouse_y);

        if self.button_resume.is_hovered {
            self.selected = Some(0);
        } else if self.button_quit.is_hovered {
            self.selected = Some(1);
        }
    }

    fn get_button(&self, pos: usize) -> &Button {
        if pos == 0 {
            &self.button_resume
        } else {
            &self.button_quit
        }
    }

    pub fn reset_buttons(&mut self) {
        self.selected = None;
        self.button_resume.is_hovered = false;
        self.button_quit.is_hovered = false;
        self.unclick_buttons();
    }

    pub fn unclick_buttons(&mut self) {
        self.button_resume.is_clicked = false;
        self.button_quit.is_clicked = false;
    }

    pub fn handle_event(&mut self, event: Event) -> MenuEvent {
        match event {
            Event::Quit { .. } => {
                self.reset_buttons();
                return MenuEvent::Quit;
            }
            // TODO: would be nice to hover buttons with keys and not just mouse
            // TODO: actually, might be worth it to just give events to the menu directly...
            Event::KeyDown {
                keycode: Some(x), ..
            } => match x {
                Keycode::Escape => {
                    self.reset_buttons();
                    return MenuEvent::Resume;
                }
                Keycode::Up => {
                    if let Some(ref mut selected) = self.selected {
                        if *selected > 0 {
                            *selected -= 1;
                        } else {
                            *selected = 1;
                        }
                    } else {
                        self.selected = Some(0);
                    }
                }
                Keycode::Down => {
                    if let Some(ref mut selected) = self.selected {
                        if *selected < 1 {
                            *selected += 1;
                        } else {
                            *selected = 0;
                        }
                    } else {
                        self.selected = Some(1);
                    }
                }
                Keycode::Return => {
                    if let Some(selected) = self.selected {
                        if selected == 0 {
                            self.reset_buttons();
                            return MenuEvent::Resume;
                        } else {
                            return MenuEvent::Quit;
                        }
                    }
                }
                _ => {}
            },
            Event::MouseMotion { x, y, .. } => {
                self.update(x, y);
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.button_resume.update_click(x, y);
                self.button_quit.update_click(x, y);
            }
            Event::MouseButtonUp {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                if self.button_resume.is_clicked && self.button_resume.is_in(x, y) {
                    self.reset_buttons();
                    return MenuEvent::Resume;
                }
                if self.button_quit.is_clicked && self.button_quit.is_in(x, y) {
                    return MenuEvent::Quit;
                }
                self.unclick_buttons();
            }
            _ => {}
        }
        MenuEvent::None
    }

    pub fn draw(&self, system: &mut System) {
        system
            .canvas
            .copy(&self.background, None, None)
            .expect("copy menu failed");
        self.button_resume.draw(system);
        self.button_quit.draw(system);
        if let Some(selected) = self.selected {
            let rect = self.get_button(selected).rect;
            system
                .canvas
                .copy(
                    &self.selected_texture,
                    None,
                    Rect::new(
                        rect.x - 30,
                        rect.y + (rect.height() - 20) as i32 / 2,
                        20,
                        20,
                    ),
                )
                .expect("copy menu failed");
        }
    }
}
