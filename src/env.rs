use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::render::TextureCreator;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use sdl2::EventPump;
use sdl2::GameControllerSubsystem;

use std::collections::HashMap;
use std::time::Instant;

use crate::character::Direction;
use crate::debug_display::DebugDisplay;
use crate::menu::{Menu, MenuEvent};
use crate::player::Player;
use crate::reward::Reward;
use crate::system::System;
use crate::texture_holder::TextureHolder;
use crate::utils::compute_distance;
use crate::{GetDimension, GetPos, FPS_REFRESH, ONE_SECOND};

pub struct Env<'a> {
    pub display_menu: bool,
    pub is_attack_pressed: bool,
    pub debug: Option<u32>,
    pub fps_str: String,
    pub debug_display: DebugDisplay<'a, 'a>,
    pub menu: Menu<'a>,
    pub need_sort_rewards: bool,
    pub closest_reward: Option<(i32, usize)>,
    pub game_controller_subsystem: &'a GameControllerSubsystem,
    pub controller: Option<GameController>,
}

impl<'a> Env<'a> {
    pub fn new(
        game_controller_subsystem: &'a GameControllerSubsystem,
        texture_creator: &'a TextureCreator<WindowContext>,
        font: &'a Font<'_, 'static>,
        width: u32,
        height: u32,
    ) -> Env<'a> {
        let mut env = Env {
            display_menu: false,
            is_attack_pressed: false,
            debug: None,
            fps_str: String::new(),
            debug_display: DebugDisplay::new(font, texture_creator, 16),
            menu: Menu::new(texture_creator, font, width, height),
            need_sort_rewards: false,
            closest_reward: None,
            game_controller_subsystem,
            controller: None,
        };
        env.update_controller();
        env
    }

    pub fn update_controller(&mut self) {
        if self.controller.is_some() {
            return;
        }
        // Enable events for the controllers.
        let available = self
            .game_controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e))
            .unwrap();

        println!("{} joysticks available", available);

        // Iterate over all available joysticks and look for game controllers.
        self.controller = (0..available).find_map(|id| {
            if !self.game_controller_subsystem.is_game_controller(id) {
                println!("{} is not a game controller", id);
                return None;
            }

            println!("Attempting to open controller {}", id);

            match self.game_controller_subsystem.open(id) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
                    println!("Success: opened \"{}\"", c.name());
                    Some(c)
                }
                Err(e) => {
                    println!("failed: {:?}", e);
                    None
                }
            }
        });
    }

    pub fn handle_events(
        &mut self,
        event_pump: &mut EventPump,
        players: &mut [Player],
        rewards: &mut Vec<Reward>,
    ) -> bool {
        let mouse_state = event_pump.mouse_state();
        for event in event_pump.poll_iter() {
            if self.display_menu {
                match self.menu.handle_event(match event {
                    Event::ControllerDeviceAdded { .. } => {
                        println!("new device detected!");
                        self.update_controller();
                        continue;
                    }
                    Event::ControllerDeviceRemoved { which, .. }
                    | Event::JoyDeviceRemoved { which, .. } => {
                        if Some(which as i32) == self.controller.as_ref().map(|c| c.instance_id()) {
                            self.controller = None;
                        }
                        println!("device removed!");
                        continue;
                    }
                    Event::ControllerButtonDown {
                        button,
                        which,
                        timestamp,
                    } if Some(which as i32)
                        == self.controller.as_ref().map(|c| c.instance_id()) =>
                    {
                        match button {
                            Button::DPadUp => Event::KeyDown {
                                keycode: Some(Keycode::Up),
                                window_id: 0,
                                timestamp,
                                scancode: None,
                                repeat: false,
                                keymod: Mod::empty(),
                            },
                            Button::DPadDown => Event::KeyDown {
                                keycode: Some(Keycode::Down),
                                window_id: 0,
                                timestamp,
                                scancode: None,
                                repeat: false,
                                keymod: Mod::empty(),
                            },
                            Button::A => Event::KeyDown {
                                keycode: Some(Keycode::Return),
                                window_id: 0,
                                timestamp,
                                scancode: None,
                                repeat: false,
                                keymod: Mod::empty(),
                            },
                            Button::B | Button::Start => Event::KeyDown {
                                keycode: Some(Keycode::Escape),
                                window_id: 0,
                                timestamp,
                                scancode: None,
                                repeat: false,
                                keymod: Mod::empty(),
                            },
                            _ => continue,
                        }
                    }
                    e => e,
                }) {
                    MenuEvent::Quit => return false,
                    MenuEvent::Resume => self.display_menu = false,
                    MenuEvent::None => {}
                }
            } else {
                match event {
                    Event::ControllerDeviceAdded { .. } => {
                        println!("new device detected!");
                        self.update_controller();
                    }
                    Event::ControllerDeviceRemoved { which, .. }
                    | Event::JoyDeviceRemoved { which, .. } => {
                        if Some(which as i32) == self.controller.as_ref().map(|c| c.instance_id()) {
                            self.controller = None;
                        }
                        println!("device removed!");
                    }
                    Event::ControllerButtonDown { button, which, .. } => {
                        if Some(which as i32) == self.controller.as_ref().map(|c| c.instance_id()) {
                            match button {
                                Button::DPadUp => players[0].handle_move(Direction::Up),
                                Button::DPadDown => players[0].handle_move(Direction::Down),
                                Button::DPadLeft => players[0].handle_move(Direction::Left),
                                Button::DPadRight => players[0].handle_move(Direction::Right),
                                Button::A => {
                                    if !self.is_attack_pressed {
                                        players[0].attack();
                                        self.is_attack_pressed = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::ControllerButtonUp { button, which, .. } => {
                        if Some(which as i32) == self.controller.as_ref().map(|c| c.instance_id()) {
                            println!("button pressed: {:?}", button);
                            match button {
                                Button::DPadUp => players[0].handle_release(Direction::Up),
                                Button::DPadDown => players[0].handle_release(Direction::Down),
                                Button::DPadLeft => players[0].handle_release(Direction::Left),
                                Button::DPadRight => players[0].handle_release(Direction::Right),
                                Button::A => self.is_attack_pressed = false,
                                Button::Start => {
                                    self.display_menu = true;
                                    self.menu.update(0, 0);
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::ControllerAxisMotion {
                        axis, which, value, ..
                    } => {
                        if Some(which as i32) == self.controller.as_ref().map(|c| c.instance_id()) {
                            match axis {
                                Axis::LeftX => {
                                    if value > 7500 {
                                        players[0].handle_move(Direction::Right)
                                    } else if value < -7500 {
                                        players[0].handle_move(Direction::Left)
                                    } else {
                                        players[0].handle_release(Direction::Right);
                                        players[0].handle_release(Direction::Left);
                                    }
                                }
                                Axis::LeftY => {
                                    if value > 7500 {
                                        players[0].handle_move(Direction::Down)
                                    } else if value < -7500 {
                                        players[0].handle_move(Direction::Up)
                                    } else {
                                        players[0].handle_release(Direction::Up);
                                        players[0].handle_release(Direction::Down);
                                    }
                                }
                                Axis::TriggerRight => {
                                    println!("{:?} {}", value > 30, !players[0].is_blocking());
                                    if value > 30 && !players[0].is_blocking() {
                                        players[0].stop_attack();
                                        players[0].block();
                                    } else if players[0].is_blocking() {
                                        players[0].stop_block();
                                        if self.is_attack_pressed {
                                            players[0].attack();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
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
                        Keycode::LCtrl => {
                            players[0].stop_attack();
                            players[0].block();
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
                        Keycode::LCtrl => {
                            players[0].stop_block();
                            if self.is_attack_pressed {
                                players[0].attack();
                            }
                        }
                        Keycode::Space => self.is_attack_pressed = false,
                        Keycode::Return => {
                            if let Some((_, reward)) = self.closest_reward.take() {
                                // TODO: actually give reward to the players[0].
                                self.need_sort_rewards = true;
                                rewards.remove(reward);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        true
    }

    pub fn debug_draw(&mut self, system: &mut System, player: &Player, elapsed: u64) {
        if let Some(ref mut debug) = self.debug {
            *debug += 1;
            if *debug >= FPS_REFRESH {
                self.fps_str = if elapsed < crate::FRAME_DELAY {
                    "FPS: 60.0".to_owned()
                } else {
                    format!("FPS: {:.2}", ONE_SECOND as f64 / elapsed as f64)
                };
                *debug = 0;
            }
            if let Some(ref stats) = player.stats {
                let total_walked = stats.borrow().get_total_walked();
                self.debug_display.draw(
                    system,
                    &format!(
                        "{}\nposition: ({}, {})\ntotal walked: {}.{:01}",
                        self.fps_str,
                        player.x(),
                        player.y(),
                        total_walked / 100,
                        total_walked % 100,
                    ),
                );
            }
        } else {
            self.debug_display.draw(system, "");
        }
    }

    pub fn draw_rewards(
        &mut self,
        system: &mut System,
        rewards: &[Reward],
        player: &Player,
        textures: &HashMap<&str, TextureHolder>,
    ) {
        if self.need_sort_rewards {
            self.closest_reward = None;
            for i in 0..rewards.len() {
                let reward = &rewards[i];
                reward.draw(system);
                match player.action.direction {
                    Direction::Up => {
                        if player.y() + 4 >= reward.y() + 4 {
                            let distance = compute_distance(player, reward);
                            if distance > 40 {
                                continue;
                            }
                            match self.closest_reward {
                                Some((ref mut dist, ref mut reward_pos)) => {
                                    if *dist > distance {
                                        *dist = distance;
                                        *reward_pos = i;
                                    }
                                }
                                None => {
                                    self.closest_reward = Some((distance, i));
                                }
                            }
                        }
                    }
                    Direction::Down => {
                        if player.height() as i64 + player.y() - 10
                            < reward.y() + reward.height() as i64
                        {
                            let distance = compute_distance(player, reward);
                            if distance > 50 {
                                continue;
                            }
                            match self.closest_reward {
                                Some((ref mut dist, ref mut reward_pos)) => {
                                    if *dist > distance {
                                        *dist = distance;
                                        *reward_pos = i;
                                    }
                                }
                                None => {
                                    self.closest_reward = Some((distance, i));
                                }
                            }
                        }
                    }
                    Direction::Right => {
                        if player.width() as i64 + player.x() - 10
                            < reward.x() + reward.width() as i64
                        {
                            let distance = compute_distance(player, reward);
                            if distance > 50 {
                                continue;
                            }
                            match self.closest_reward {
                                Some((ref mut dist, ref mut reward_pos)) => {
                                    if *dist > distance {
                                        *dist = distance;
                                        *reward_pos = i;
                                    }
                                }
                                None => {
                                    self.closest_reward = Some((distance, i));
                                }
                            }
                        }
                    }
                    Direction::Left => {
                        if player.x() > reward.x() + 4 {
                            let distance = compute_distance(player, reward);
                            if distance > 50 {
                                continue;
                            }
                            match self.closest_reward {
                                Some((ref mut dist, ref mut reward_pos)) => {
                                    if *dist > distance {
                                        *dist = distance;
                                        *reward_pos = i;
                                    }
                                }
                                None => {
                                    self.closest_reward = Some((distance, i));
                                }
                            }
                        }
                    }
                }
            }
            self.need_sort_rewards = false;
        } else {
            for reward in rewards.iter() {
                reward.draw(system);
            }
        }
        if let Some((_, pos)) = self.closest_reward {
            let reward = &rewards[pos];
            let texture = &textures["reward-text"];
            texture.draw(
                system,
                reward.x() + (reward.width() as i64) / 2 - (texture.width as i64) / 2,
                reward.y() - 2 - texture.height as i64,
            );
        }
    }
}
