use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureCreator;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use sdl2::EventPump;

use std::time::Instant;

use crate::character::Direction;
use crate::debug_display::DebugDisplay;
use crate::menu::{Menu, MenuEvent};
use crate::player::Player;
use crate::system::System;
use crate::{GetPos, FPS_REFRESH, ONE_SECOND};

pub struct Env<'a> {
    pub display_menu: bool,
    pub is_attack_pressed: bool,
    pub debug: Option<u32>,
    pub fps_str: String,
    pub debug_display: DebugDisplay<'a, 'a>,
    pub menu: Menu<'a>,
}

impl<'a> Env<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'a Font<'_, 'static>,
        width: u32,
        height: u32,
    ) -> Env<'a> {
        Env {
            display_menu: false,
            is_attack_pressed: false,
            debug: None,
            fps_str: String::new(),
            debug_display: DebugDisplay::new(font, texture_creator, 16),
            menu: Menu::new(texture_creator, font, width, height),
        }
    }

    pub fn handle_events(&mut self, event_pump: &mut EventPump, players: &mut [Player]) -> bool {
        let mouse_state = event_pump.mouse_state();
        for event in event_pump.poll_iter() {
            if self.display_menu {
                match self.menu.handle_event(event) {
                    MenuEvent::Quit => return false,
                    MenuEvent::Resume => self.display_menu = false,
                    MenuEvent::None => {}
                }
            } else {
                match event {
                    Event::Quit { .. } => return false,
                    Event::KeyDown {
                        keycode: Some(x), ..
                    } => match x {
                        Keycode::Escape => {
                            self.display_menu = true;
                            // To hover buttons in case the mouse is hovering one.
                            self.menu.update(mouse_state.x(), mouse_state.y());
                        }
                        Keycode::Left | Keycode::Q => players[0].handle_move(Direction::Left),
                        Keycode::Right | Keycode::D => players[0].handle_move(Direction::Right),
                        Keycode::Up | Keycode::Z => players[0].handle_move(Direction::Up),
                        Keycode::Down | Keycode::S => players[0].handle_move(Direction::Down),
                        Keycode::Space => {
                            if !self.is_attack_pressed {
                                players[0].attack();
                                self.is_attack_pressed = true;
                            }
                        }
                        Keycode::LShift => {
                            players[0].is_run_pressed = true;
                            players[0].is_running = players[0].action.movement.is_some();
                        }
                        Keycode::F3 => {
                            if self.debug.is_some() {
                                self.debug = None;
                            } else {
                                self.debug = Some(FPS_REFRESH - 1);
                            }
                        }
                        Keycode::F5 => self.debug_display.switch_draw_grid(),
                        _ => {}
                    },
                    Event::KeyUp {
                        keycode: Some(x), ..
                    } => match x {
                        Keycode::Left | Keycode::Q => players[0].handle_release(Direction::Left),
                        Keycode::Right | Keycode::D => players[0].handle_release(Direction::Right),
                        Keycode::Up | Keycode::Z => players[0].handle_release(Direction::Up),
                        Keycode::Down | Keycode::S => players[0].handle_release(Direction::Down),
                        Keycode::LShift => {
                            players[0].is_run_pressed = false;
                            players[0].is_running = false;
                        }
                        Keycode::Space => self.is_attack_pressed = false,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        true
    }

    pub fn debug_draw(&mut self, system: &mut System, player: &Player, loop_timer: &Instant) {
        if let Some(ref mut debug) = self.debug {
            *debug += 1;
            if *debug >= FPS_REFRESH {
                let elapsed_time = loop_timer.elapsed();
                self.fps_str = format!(
                    "FPS: {:.2}",
                    ONE_SECOND as f64 / elapsed_time.as_micros() as f64
                );
                *debug = 0;
            }
            self.debug_display.draw(
                system,
                &format!(
                    "{}\nposition: ({}, {})",
                    self.fps_str,
                    player.x(),
                    player.y()
                ),
            );
        } else {
            self.debug_display.draw(system, "");
        }
    }
}
