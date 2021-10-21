use crate::sdl2::event::Event;
use crate::sdl2::keyboard::Keycode;
use crate::sdl2::mouse::MouseButton;
use crate::sdl2::pixels::{Color, PixelFormatEnum};
use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::system::System;
use crate::texture_holder::{TextureHolder, TextureId, Textures};

struct Button {
    texture: TextureId,
    texture_clicked: TextureId,
    text: String,
    rect: Rect,
    is_hovered: bool,
    is_clicked: bool,
    action: MenuEvent,
}

impl Button {
    fn new(
        text: String,
        button_texture: TextureId,
        button_texture_clicked: TextureId,
        rect: Rect,
        action: MenuEvent,
    ) -> Button {
        Button {
            texture: button_texture,
            texture_clicked: button_texture_clicked,
            text,
            rect,
            is_hovered: false,
            is_clicked: false,
            action,
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
        system.copy_to_canvas(
            if self.is_clicked {
                self.texture_clicked
            } else {
                self.texture
            },
            None,
            self.rect,
        );
        system.draw_text(
            &self.text,
            16,
            if self.is_hovered {
                Color::RGB(74, 138, 221)
            } else {
                Color::RGB(255, 255, 255)
            },
            self.rect.x + (self.rect.width() / 2) as i32,
            self.rect.y + (self.rect.height() / 2) as i32,
            true,
            true,
        );
    }
}

const MENUS: once_cell::sync::Lazy<Vec<(&'static str, Vec<(&'static str, MenuEvent)>)>> =
    once_cell::sync::Lazy::new(|| {
        vec![
            (
                "start",
                vec![
                    ("Start", MenuEvent::StartGame),
                    ("Settings", MenuEvent::GoTo("settings")),
                    ("Quit", MenuEvent::Quit),
                ],
            ),
            (
                "death",
                vec![
                    ("Resurrect", MenuEvent::Resurrect),
                    ("Save and exit", MenuEvent::Quit),
                ],
            ),
            (
                "pause",
                vec![
                    ("Resume", MenuEvent::Resume),
                    ("Settings", MenuEvent::GoTo("settings")),
                    ("Quit", MenuEvent::Quit),
                ],
            ),
            (
                "settings",
                vec![("Stuff", MenuEvent::None), ("Back", MenuEvent::GoBack)],
            ),
        ]
    });

#[derive(Clone, Copy, Debug)]
pub enum MenuEvent {
    StartGame,
    Quit,
    Resume,
    Resurrect,
    GoBack,
    GoTo(&'static str),
    None,
}

pub struct Menu {
    background: TextureId,
    buttons: Vec<Button>,
    selected: Option<usize>,
    selected_texture: TextureId,
    parent_state: Vec<&'static str>,
    state: &'static str,
    width: u32,
    height: u32,
}

impl Menu {
    pub fn init_button_textures<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
        width: u32,
        height: u32,
    ) {
        let mut button = Surface::new(width, height, texture_creator.default_pixel_format())
            .expect("failed to create button surface");
        button
            .fill_rect(None, Color::RGB(30, 30, 30))
            .expect("failed to fill button surface");
        textures.add_named_texture(
            "t:button",
            TextureHolder::surface_into_texture(texture_creator, button),
        );

        let mut button_clicked =
            Surface::new(width, height, texture_creator.default_pixel_format())
                .expect("failed to create button surface");
        button_clicked
            .fill_rect(None, Color::RGB(20, 20, 20))
            .expect("failed to fill button surface");
        textures.add_named_texture(
            "t:button-clicked",
            TextureHolder::surface_into_texture(texture_creator, button_clicked),
        );
    }

    pub fn new<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
        width: u32,
        height: u32,
    ) -> Self {
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
            background: textures.add_texture(TextureHolder::surface_into_texture(
                texture_creator,
                background,
            )),
            buttons: Vec::new(),
            selected_texture: textures.add_texture(TextureHolder::surface_into_texture(
                texture_creator,
                selected_surface,
            )),
            selected: None,
            state: "",
            parent_state: Vec::new(),
            width,
            height,
        }
    }

    pub fn set_pause(&mut self, textures: &Textures<'_>) {
        self.parent_state.clear();
        self.set_state("pause", textures);
        self.update(0, 0);
    }

    pub fn set_death(&mut self, textures: &Textures<'_>) {
        self.parent_state.clear();
        self.set_state("death", textures);
        self.update(0, 0);
    }

    pub fn set_state(&mut self, state: &'static str, textures: &Textures<'_>) {
        if self.state == state {
            return;
        }
        if self.state != "" {
            self.parent_state.push(self.state);
            self.selected = Some(0);
        } else {
            self.selected = None;
        }
        self.state = state;
        for (name, buttons) in MENUS.iter() {
            if *name == self.state {
                self.buttons.clear();
                let total = buttons.len() as i32 + 1;
                for (pos, (text, action)) in buttons.iter().enumerate() {
                    let button_texture = textures.get_texture_id_from_name("t:button");
                    let button_texture_clicked =
                        textures.get_texture_id_from_name("t:button-clicked");
                    self.buttons.push(Button::new(
                        text.to_string(),
                        button_texture,
                        button_texture_clicked,
                        Rect::new(
                            self.width as i32 / 4,
                            // 25 is the button's height divided by 2
                            self.height as i32 / total * (pos + 1) as i32 - 25,
                            self.width / 2,
                            50,
                        ),
                        *action,
                    ));
                }
            }
        }
    }

    pub fn update(&mut self, mouse_x: i32, mouse_y: i32) {
        for (pos, button) in self.buttons.iter_mut().enumerate() {
            button.update(mouse_x, mouse_y);
            if button.is_hovered {
                self.selected = Some(pos);
            }
        }
    }

    fn get_selected_button(&self) -> Option<&Button> {
        match self.selected {
            Some(selected) => self.buttons.get(selected),
            None => None,
        }
    }

    pub fn reset_buttons(&mut self) {
        self.selected = None;
        for button in self.buttons.iter_mut() {
            button.is_hovered = false;
        }
        self.unclick_buttons();
    }

    pub fn unclick_buttons(&mut self) {
        for button in self.buttons.iter_mut() {
            button.is_clicked = false;
        }
    }

    fn handle_button(&mut self, button_pos: usize, textures: &Textures<'_>) -> MenuEvent {
        match self.buttons[button_pos].action {
            MenuEvent::StartGame => return MenuEvent::Resume,
            MenuEvent::GoBack => {
                self.state = "";
                if let Some(p) = self.parent_state.pop() {
                    self.set_state(p, textures);
                    return MenuEvent::None;
                } else {
                    return MenuEvent::Resume;
                }
            }
            MenuEvent::GoTo(s) => {
                self.set_state(s, textures);
                return MenuEvent::None;
            }
            e => return e,
        }
    }

    pub fn handle_event(&mut self, event: Event, textures: &Textures<'_>) -> MenuEvent {
        match event {
            Event::Quit { .. } => {
                self.reset_buttons();
                return MenuEvent::Quit;
            }
            Event::KeyDown {
                keycode: Some(x), ..
            } => match x {
                Keycode::Escape => {
                    self.state = "";
                    if let Some(p) = self.parent_state.pop() {
                        self.set_state(p, textures);
                        return MenuEvent::None;
                    } else {
                        self.reset_buttons();
                        return MenuEvent::Resume;
                    }
                }
                Keycode::Up => {
                    if let Some(ref mut selected) = self.selected {
                        if *selected > 0 {
                            *selected -= 1;
                        } else {
                            *selected = self.buttons.len() - 1;
                        }
                    } else {
                        self.selected = Some(0);
                    }
                }
                Keycode::Down => {
                    if let Some(ref mut selected) = self.selected {
                        if *selected + 1 < self.buttons.len() {
                            *selected += 1;
                        } else {
                            *selected = 0;
                        }
                    } else {
                        self.selected = Some(self.buttons.len() - 1);
                    }
                }
                Keycode::Return => match self.selected {
                    Some(selected) => return self.handle_button(selected, textures),
                    None => {}
                },
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
                for button in self.buttons.iter_mut() {
                    button.update_click(x, y);
                }
            }
            Event::MouseButtonUp {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                let clicked = self
                    .buttons
                    .iter()
                    .position(|b| b.is_clicked && b.is_in(x, y));
                self.unclick_buttons();
                if let Some(clicked) = clicked {
                    return self.handle_button(clicked, textures);
                }
            }
            _ => {}
        }
        MenuEvent::None
    }

    pub fn draw(&self, system: &mut System) {
        system.copy_to_canvas(self.background, None, None);
        for button in self.buttons.iter() {
            button.draw(system);
        }
        if let Some(selected) = self.get_selected_button() {
            let rect = selected.rect;
            system.copy_to_canvas(
                self.selected_texture,
                None,
                Rect::new(
                    rect.x - 30,
                    rect.y + (rect.height() - 20) as i32 / 2,
                    20,
                    20,
                ),
            );
        }
    }
}
