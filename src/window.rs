use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use std::collections::HashMap;

use crate::system::System;
use crate::texture_holder::TextureHolder;
use crate::GetDimension;

pub trait Widget: GetDimension {
    fn draw(&self, system: &mut System, x_add: i32, y_add: i32);
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn is_in(&self, x: i32, y: i32) -> bool {
        !(x < self.x()
            || x > self.x() + self.width() as i32
            || y < self.y()
            || y > self.y() + self.height() as i32)
    }
    /// Returns true is the event has been handled
    fn handle_event(&mut self, ev: &Event, x_add: i32, y_add: i32) -> Option<EventAction>;
}

#[derive(Clone, Copy, PartialEq)]
enum EventAction {
    None,
    Close,
}

struct TitleBarButton<'a> {
    texture: Texture<'a>,
    texture_pressed: Texture<'a>,
    texture_hovered: Texture<'a>,
    texture_hovered_and_pressed: Texture<'a>,
    size: u32,
    is_hovered: bool,
    is_pressed: bool,
    x: i32,
    y: i32,
}

impl<'a> TitleBarButton<'a> {
    fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        size: u32,
        x: i32,
        y: i32,
    ) -> TitleBarButton<'a> {
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
            x,
            y,
        }
    }
}

impl<'a> GetDimension for TitleBarButton<'a> {
    fn width(&self) -> u32 {
        self.size
    }
    fn height(&self) -> u32 {
        self.size
    }
}

impl<'a> Widget for TitleBarButton<'a> {
    fn draw(&self, system: &mut System, x_add: i32, y_add: i32) {
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
            .copy(
                t,
                None,
                Rect::new(self.x + x_add, self.y + y_add, self.size, self.size),
            )
            .expect("failed to draw titlebar button");
    }
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn handle_event(&mut self, ev: &Event, x_add: i32, y_add: i32) -> Option<EventAction> {
        if !ev.is_mouse() && !ev.is_controller() {
            return None;
        }
        match ev {
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x: mouse_x,
                y: mouse_y,
                ..
            } => {
                if self.is_in(*mouse_x - x_add, *mouse_y - y_add) {
                    self.is_pressed = true;
                    Some(EventAction::None)
                } else {
                    None
                }
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x: mouse_x,
                y: mouse_y,
                ..
            } => {
                let ret = if self.is_pressed && self.is_in(*mouse_x - x_add, *mouse_y - y_add) {
                    self.is_hovered = true;
                    Some(EventAction::Close)
                } else {
                    None
                };
                self.is_pressed = false;
                ret
            }
            Event::MouseMotion {
                x: mouse_x,
                y: mouse_y,
                ..
            } => {
                self.is_hovered = self.is_in(*mouse_x - x_add, *mouse_y - y_add);
                if self.is_hovered {
                    Some(EventAction::None)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct InventoryCase<'a> {
    texture_hovered: &'a TextureHolder<'a>,
    x: i32,
    y: i32,
    is_hovered: bool,
}

impl<'a> InventoryCase<'a> {
    pub fn init_textures(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut HashMap<&'static str, TextureHolder<'a>>,
        width: u32,
    ) {
        let size = InventoryCases::get_case_size(width - 2);

        let mut inventory_case =
            Surface::new(size - 4, size - 4, texture_creator.default_pixel_format())
                .expect("failed to create inventory case surface");
        inventory_case
            .fill_rect(None, Color::RGB(250, 183, 55))
            .expect("failed to fill inventory case");
        textures.insert(
            "inventory-case-hover",
            TextureHolder::surface_to_texture(texture_creator, inventory_case),
        );
    }
    fn new(
        textures: &'a HashMap<&'static str, TextureHolder<'a>>,
        x: i32,
        y: i32,
    ) -> InventoryCase<'a> {
        InventoryCase {
            x,
            y,
            texture_hovered: &textures["inventory-case-hover"],
            is_hovered: false,
        }
    }
}

impl<'a> GetDimension for InventoryCase<'a> {
    fn width(&self) -> u32 {
        self.texture_hovered.width
    }
    fn height(&self) -> u32 {
        self.texture_hovered.height
    }
}

impl<'a> Widget for InventoryCase<'a> {
    fn draw(&self, system: &mut System, x_add: i32, y_add: i32) {
        if !self.is_hovered {
            return;
        }
        system
            .canvas
            .copy(
                &self.texture_hovered.texture,
                None,
                Rect::new(
                    self.x + x_add + 2,
                    self.y + y_add + 2,
                    self.texture_hovered.width,
                    self.texture_hovered.height,
                ),
            )
            .expect("failed to draw titlebar button");
    }
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn handle_event(&mut self, ev: &Event, x_add: i32, y_add: i32) -> Option<EventAction> {
        match ev {
            Event::MouseMotion {
                x: mouse_x,
                y: mouse_y,
                ..
            } => {
                self.is_hovered = self.is_in(*mouse_x - x_add, *mouse_y - y_add);
                if self.is_hovered {
                    Some(EventAction::None)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct InventoryCases<'a> {
    texture: TextureHolder<'a>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    widgets: Vec<InventoryCase<'a>>,
}

impl<'a> InventoryCases<'a> {
    fn get_case_size(width: u32) -> u32 {
        // We want 4 inventory cases per line, so it makes "5 borders".
        (width - 3) / 4
    }
    fn new(
        textures: &'a HashMap<&'static str, TextureHolder<'a>>,
        texture_creator: &'a TextureCreator<WindowContext>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> InventoryCases<'a> {
        let size = Self::get_case_size(width - 2);
        let mut inventory_case = Surface::new(size, size, texture_creator.default_pixel_format())
            .expect("failed to create inventory case surface");
        inventory_case
            .fill_rect(None, Color::RGB(0, 0, 0))
            .expect("failed to fill inventory case");
        inventory_case
            .fill_rect(
                Rect::new(2, 2, size - 4, size - 4),
                Color::RGB(200, 140, 27),
            )
            .expect("failed to fill inventory case");
        let nb_rows = 6;
        let total_height = (size + 1) * nb_rows + 1;
        let mut surface = Surface::new(width - 2, total_height, PixelFormatEnum::RGBA8888)
            .expect("failed to create cases surface");
        let mut widgets = Vec::with_capacity((4 * total_height / size) as usize);
        let mut pos_y = 0;
        for _ in 0..nb_rows {
            let mut pos_x = 0;
            for _ in 0..4 {
                inventory_case
                    .blit(None, &mut surface, Rect::new(pos_x, pos_y, size, size))
                    .expect("failed to blit surface case");
                widgets.push(InventoryCase::new(textures, pos_x + x, pos_y + y));
                pos_x += size as i32 + 1;
            }
            pos_y += size as i32 + 1;
        }
        InventoryCases {
            texture: TextureHolder::surface_to_texture(texture_creator, surface),
            x,
            y,
            height,
            width,
            widgets,
        }
    }
}

impl<'a> GetDimension for InventoryCases<'a> {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}

impl<'a> Widget for InventoryCases<'a> {
    fn draw(&self, system: &mut System, x_add: i32, y_add: i32) {
        system
            .canvas
            .copy(
                &self.texture.texture,
                Rect::new(0, 0, self.texture.width, self.height - 2),
                Rect::new(
                    self.x + x_add + 1,
                    self.y + y_add + 1,
                    self.texture.width,
                    self.height - 2,
                ),
            )
            .expect("failed to draw inventory cases");
        for widget in self.widgets.iter() {
            widget.draw(system, x_add + 1, y_add + 1);
        }
    }
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn handle_event(&mut self, ev: &Event, x_add: i32, y_add: i32) -> Option<EventAction> {
        let mut ret = None;
        for widget in self.widgets.iter_mut() {
            match ev {
                Event::MouseWheel { .. } => {
                    // handle scroll
                }
                e => {
                    if let Some(r) = widget.handle_event(e, x_add, y_add) {
                        ret = Some(r);
                    }
                }
            }
        }
        ret
    }
}

pub fn create_inventory_window<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    textures: &'a HashMap<&'static str, TextureHolder<'a>>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    border_width: u32,
) -> Window<'a> {
    let mut w = Window::new(
        texture_creator,
        textures,
        x,
        y,
        width,
        height,
        "Inventory",
        border_width,
    );
    w.widgets.push(Box::new(InventoryCases::new(
        textures,
        texture_creator,
        border_width as i32,
        w.title_bar_height as i32,
        width - border_width * 2,
        height - w.title_bar_height - border_width,
    )));
    w
}

pub struct Window<'a> {
    title_bar_height: u32,
    pub title: &'static str,
    border_width: u32,
    texture: TextureHolder<'a>,
    x: i32,
    y: i32,
    is_hidden: bool,
    is_dragging_window: Option<(i32, i32)>,
    widgets: Vec<Box<Widget + 'a>>,
}

impl<'a> Window<'a> {
    pub fn init_textures(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut HashMap<&'static str, TextureHolder<'a>>,
        width: u32,
        border_width: u32,
    ) {
        InventoryCase::init_textures(texture_creator, textures, width - border_width * 2);
    }
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &'a HashMap<&'static str, TextureHolder<'a>>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: &'static str,
        border_width: u32,
    ) -> Window<'a> {
        let title_bar_height = 22;
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
            widgets: vec![Box::new(TitleBarButton::new(
                texture_creator,
                title_bar_height - 6,
                width as i32 - title_bar_height as i32 + 3,
                3,
            ))],
            title,
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
        for widget in self.widgets.iter() {
            widget.draw(system, self.x, self.y);
        }
        system.draw_text(
            self.title,
            16,
            Color::RGB(255, 255, 255),
            self.x + self.texture.width as i32 / 2,
            self.y + self.title_bar_height as i32 / 2,
            true,
            true,
        );
    }
    fn is_in(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x <= self.x + self.width() as i32
            && y >= self.y
            && y <= self.y + self.height() as i32
    }
    fn is_in_title_bar(&self, x: i32, y: i32) -> bool {
        self.is_in(x, y) && y <= self.y + self.title_bar_height as i32
    }
    /// Returns `true` if the event has been handled by the window (i.e. if it affected itself or
    /// one of its widgets).
    pub fn handle_event(&mut self, ev: &Event) -> bool {
        if self.is_hidden || (!ev.is_mouse() && !ev.is_controller()) {
            return false;
        }
        match ev {
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x: mouse_x,
                y: mouse_y,
                ..
            } => {
                // TODO: clean this up
                let ev = Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x: *mouse_x,
                    y: *mouse_y,
                    timestamp: 0,
                    which: 0,
                    clicks: 0,
                    window_id: 0,
                };
                let mut actions = false;
                for widget in self.widgets.iter_mut() {
                    if widget.handle_event(&ev, self.x, self.y).is_some() {
                        actions = true;
                    }
                }
                // If we are in the titlebar, then we can drag the window.
                if !actions && self.is_in_title_bar(*mouse_x, *mouse_y) {
                    self.is_dragging_window = Some((*mouse_x - self.x, *mouse_y - self.y));
                    true
                } else {
                    actions
                }
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x: mouse_x,
                y: mouse_y,
                ..
            } => {
                let mut was_handled = self.is_dragging_window.is_some();
                self.is_dragging_window = None;
                if self.is_in(*mouse_x, *mouse_y) {
                    // TODO: clean this up
                    let ev = Event::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        x: *mouse_x,
                        y: *mouse_y,
                        timestamp: 0,
                        which: 0,
                        clicks: 0,
                        window_id: 0,
                    };
                    for widget in self.widgets.iter_mut() {
                        match widget.handle_event(&ev, self.x, self.y) {
                            Some(e) => {
                                if e == EventAction::Close {
                                    self.is_hidden = true;
                                }
                                was_handled = true;
                            }
                            None => {}
                        }
                    }
                }
                was_handled
            }
            Event::MouseMotion {
                x: mouse_x,
                y: mouse_y,
                mousestate,
                ..
            } => match self.is_dragging_window {
                Some((x_add, y_add)) => {
                    self.x = *mouse_x - x_add;
                    self.y = *mouse_y - y_add;
                    true
                }
                None => {
                    let mut was_handled = false;
                    if self.is_in(*mouse_x, *mouse_y) {
                        // TODO: clean this up
                        let ev = Event::MouseMotion {
                            x: *mouse_x,
                            y: *mouse_y,
                            xrel: 0,
                            yrel: 0,
                            timestamp: 0,
                            which: 0,
                            mousestate: *mousestate,
                            window_id: 0,
                        };
                        for widget in self.widgets.iter_mut() {
                            if widget.handle_event(&ev, self.x, self.y).is_some() {
                                was_handled = true;
                            }
                        }
                    }
                    was_handled
                }
            },
            _ => false,
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
