use crate::sdl2::controller::{Axis, Button, GameController};
use crate::sdl2::event::Event;
use crate::sdl2::keyboard::{Keycode, Mod};
use crate::sdl2::render::TextureCreator;
use crate::sdl2::video::WindowContext;
use crate::sdl2::EventPump;
use crate::sdl2::GameControllerSubsystem;

use crate::character::Direction;
use crate::debug_display::DebugDisplay;
use crate::menu::{Menu, MenuEvent};
use crate::player::Player;
use crate::reward::Reward;
use crate::system::System;
use crate::texture_holder::{TextureId, Textures};
use crate::utils::compute_distance;
use crate::{GetDimension, GetPos, ONE_SECOND};

#[derive(Clone, Copy, Debug, PartialEq)]
enum EventKind {
    Press,
    Release,
}

struct GamePad {
    controller: GameController,
    left_stick_last_event_x: (EventKind, Direction),
    left_stick_last_event_y: (EventKind, Direction),
    right_stick_last_event_x: (EventKind, Direction),
    right_stick_last_event_y: (EventKind, Direction),
    left_trigger_last_event: EventKind,
    right_trigger_last_event: EventKind,
}

impl Direction {
    fn into_sdl_keycode(self) -> Keycode {
        match self {
            Direction::Right => Keycode::Right,
            Direction::Left => Keycode::Left,
            Direction::Up => Keycode::Up,
            Direction::Down => Keycode::Down,
        }
    }
}

macro_rules! update_axis {
    ($field:expr, $value:ident, $dir1:expr, $dir2:expr, $timestamp:expr) => {{
        let is_pressed = $field.0 == EventKind::Press;
        let mut tmp = None;
        if $value > 7500 {
            if $field == (EventKind::Press, $dir1) {
                return vec![];
            }
            if is_pressed {
                tmp = Some($dir2);
            }
            $field = (EventKind::Press, $dir1);
        } else if $value < -7500 {
            if $field == (EventKind::Press, $dir2) {
                return vec![];
            }
            if is_pressed {
                tmp = Some($dir2);
            }
            $field = (EventKind::Press, $dir2);
        } else {
            if $field.0 == EventKind::Release {
                return vec![];
            }
            $field.0 = EventKind::Release;
            return vec![Event::KeyUp {
                keycode: Some($field.1.into_sdl_keycode()),
                window_id: 0,
                timestamp: $timestamp,
                scancode: None,
                repeat: false,
                keymod: Mod::empty(),
            }];
        }
        if let Some(tmp) = tmp {
            vec![
                Event::KeyUp {
                    keycode: Some(tmp.into_sdl_keycode()),
                    window_id: 0,
                    timestamp: $timestamp,
                    scancode: None,
                    repeat: false,
                    keymod: Mod::empty(),
                },
                Event::KeyDown {
                    keycode: Some($field.1.into_sdl_keycode()),
                    window_id: 0,
                    timestamp: $timestamp,
                    scancode: None,
                    repeat: false,
                    keymod: Mod::empty(),
                },
            ]
        } else {
            vec![Event::KeyDown {
                keycode: Some($field.1.into_sdl_keycode()),
                window_id: 0,
                timestamp: $timestamp,
                scancode: None,
                repeat: false,
                keymod: Mod::empty(),
            }]
        }
    }};
}

macro_rules! update_trigger {
    ($keycode:expr, $field:expr, $timestamp:expr, $value:expr) => {{
        if $value > 30 {
            if $field != EventKind::Press {
                $field = EventKind::Press;
                vec![Event::KeyDown {
                    keycode: Some($keycode),
                    window_id: 0,
                    timestamp: $timestamp,
                    scancode: None,
                    repeat: false,
                    keymod: Mod::empty(),
                }]
            } else {
                vec![]
            }
        } else {
            if $field == EventKind::Press {
                $field = EventKind::Release;
                vec![Event::KeyUp {
                    keycode: Some($keycode),
                    window_id: 0,
                    timestamp: $timestamp,
                    scancode: None,
                    repeat: false,
                    keymod: Mod::empty(),
                }]
            } else {
                vec![]
            }
        }
    }};
}

impl GamePad {
    pub fn new(controller: GameController) -> Self {
        GamePad {
            controller,
            left_stick_last_event_x: (EventKind::Release, Direction::Up),
            left_stick_last_event_y: (EventKind::Release, Direction::Up),
            right_stick_last_event_x: (EventKind::Release, Direction::Up),
            right_stick_last_event_y: (EventKind::Release, Direction::Up),
            left_trigger_last_event: EventKind::Release,
            right_trigger_last_event: EventKind::Release,
        }
    }
    pub fn convert_event(&mut self, event: Event) -> Vec<Event> {
        // TODO: ignore controller events and instead get buttons and joysticks state.
        match event {
            Event::ControllerAxisMotion { which, .. }
            | Event::ControllerButtonDown { which, .. }
            | Event::ControllerButtonUp { which, .. }
            | Event::ControllerDeviceRemoved { which, .. }
            | Event::JoyDeviceRemoved { which, .. }
                if which != self.controller.instance_id() =>
            {
                vec![]
            }
            Event::ControllerAxisMotion {
                timestamp,
                axis,
                value,
                ..
            } => match axis {
                Axis::LeftX => update_axis!(
                    self.left_stick_last_event_x,
                    value,
                    Direction::Right,
                    Direction::Left,
                    timestamp
                ),
                Axis::LeftY => update_axis!(
                    self.left_stick_last_event_y,
                    value,
                    Direction::Down,
                    Direction::Up,
                    timestamp
                ),
                Axis::RightX => update_axis!(
                    self.right_stick_last_event_x,
                    value,
                    Direction::Right,
                    Direction::Left,
                    timestamp
                ),
                Axis::RightY => update_axis!(
                    self.right_stick_last_event_y,
                    value,
                    Direction::Down,
                    Direction::Up,
                    timestamp
                ),
                Axis::TriggerRight => update_trigger!(
                    Keycode::LCtrl,
                    self.right_trigger_last_event,
                    timestamp,
                    value
                ),
                Axis::TriggerLeft => update_trigger!(
                    Keycode::LShift,
                    self.left_trigger_last_event,
                    timestamp,
                    value
                ),
                // _ => vec![Event::ControllerAxisMotion {
                //     axis,
                //     timestamp,
                //     value,
                //     which,
                // }],
            },
            Event::ControllerDeviceRemoved { which, timestamp }
            | Event::JoyDeviceRemoved { which, timestamp } => {
                // We remap the two events into one.
                vec![Event::ControllerDeviceRemoved { which, timestamp }]
            }
            ev => vec![ev],
        }
    }
}

#[derive(Default)]
pub struct EguiWindow {
    pub has_focus: bool,
    pub is_displayed: bool,
}

pub struct Env<'a> {
    pub display_menu: bool,
    pub is_attack_pressed: bool,
    pub debug: bool,
    pub fps_str: String,
    pub debug_display: DebugDisplay,
    pub menu: Menu,
    pub need_sort_rewards: bool,
    pub closest_reward: Option<(i32, usize)>,
    pub game_controller_subsystem: &'a GameControllerSubsystem,
    controller: Option<GamePad>,
    // pressed_keys: Vec<Event>,
    pub character_window: EguiWindow,
    pub inventory_window: EguiWindow,
    reward_text: TextureId,
}

impl<'a> Env<'a> {
    pub fn init_textures<'b>(
        textures: &mut Textures<'b>,
        texture_creator: &'b TextureCreator<WindowContext>,
        width: u32,
        height: u32,
    ) {
        Menu::init_button_textures(texture_creator, textures, width, height);
        // Window::init_textures(texture_creator, textures, WINDOW_WIDTH, 1);
    }

    pub fn new<'b>(
        game_controller_subsystem: &'a GameControllerSubsystem,
        texture_creator: &'b TextureCreator<WindowContext>,
        textures: &mut Textures<'b>,
        width: u32,
        height: u32,
    ) -> Self {
        let mut env = Env {
            display_menu: false,
            is_attack_pressed: false,
            debug: false,
            fps_str: String::new(),
            debug_display: DebugDisplay::new(texture_creator, textures, 16),
            menu: Menu::new(texture_creator, textures, width, height),
            need_sort_rewards: false,
            closest_reward: None,
            game_controller_subsystem,
            controller: None,
            character_window: Default::default(),
            inventory_window: Default::default(),
            reward_text: textures.get_texture_id_from_name("reward-text"),
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
                    Some(GamePad::new(c))
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
        textures: &Textures<'_>,
        egui_input_state: &mut egui_sdl2_gl::EguiInputState,
    ) -> bool {
        let mouse_state = event_pump.mouse_state();
        for event in event_pump.poll_iter() {
            let events = match self.controller {
                Some(ref mut c) => c.convert_event(event),
                None => vec![event],
            };
            for event in events {
                if self.display_menu {
                    match self.menu.handle_event(
                        match event {
                            Event::ControllerDeviceAdded { .. } => {
                                println!("new device detected!");
                                self.update_controller();
                                continue;
                            }
                            Event::ControllerDeviceRemoved { which, .. } => {
                                if self
                                    .controller
                                    .as_ref()
                                    .map(|c| c.controller.instance_id() == which)
                                    .unwrap_or(false)
                                {
                                    self.controller = None;
                                    println!("device removed!");
                                }
                                continue;
                            }
                            Event::ControllerButtonDown {
                                button, timestamp, ..
                            } => match button {
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
                            },
                            e => e,
                        },
                        textures,
                    ) {
                        MenuEvent::Quit => return false,
                        MenuEvent::Resume => self.display_menu = false,
                        // TODO: better handling for resurrection:
                        // * reset monsters
                        // * reset resources
                        // * reset player position
                        //
                        // Keep in mind that in multiplayer mode, only reset everything if both
                        // players die!
                        MenuEvent::Resurrect => {
                            players[0].resurrect();
                            self.display_menu = false;
                        }
                        MenuEvent::None => {}
                        _ => {}
                    }
                } else {
                    match event {
                        Event::ControllerDeviceAdded { .. } => {
                            println!("new device detected!");
                            self.update_controller();
                        }
                        Event::ControllerDeviceRemoved { .. } => {
                            self.controller = None;
                        }
                        Event::ControllerButtonDown { button, .. } => match button {
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
                        },
                        Event::ControllerButtonUp { button, .. } => match button {
                            Button::DPadUp => players[0].handle_release(Direction::Up),
                            Button::DPadDown => players[0].handle_release(Direction::Down),
                            Button::DPadLeft => players[0].handle_release(Direction::Left),
                            Button::DPadRight => players[0].handle_release(Direction::Right),
                            Button::A => self.is_attack_pressed = false,
                            Button::Start => {
                                self.display_menu = true;
                                self.menu.set_pause(&textures);
                            }
                            _ => {}
                        },
                        Event::Quit { .. } => return false,
                        Event::KeyDown {
                            keycode: Some(x), ..
                        } => match x {
                            Keycode::Escape => {
                                let all_hidden = !self.character_window.is_displayed
                                    && !self.inventory_window.is_displayed;
                                if all_hidden {
                                    self.display_menu = true;
                                    self.menu.set_pause(textures);
                                    // To hover buttons in case the mouse is hovering one.
                                    self.menu.update(mouse_state.x(), mouse_state.y());
                                } else if self.character_window.is_displayed {
                                    self.character_window.is_displayed = false;
                                } else if self.inventory_window.is_displayed {
                                    self.inventory_window.is_displayed = false;
                                }
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
                                self.debug = self.debug == false;
                            }
                            Keycode::F5 => self.debug_display.switch_draw_grid(),
                            Keycode::I => {
                                self.inventory_window.is_displayed =
                                    !self.inventory_window.is_displayed
                            }
                            Keycode::C => {
                                self.character_window.is_displayed =
                                    !self.character_window.is_displayed
                            }
                            _ => {}
                        },
                        Event::KeyUp {
                            keycode: Some(x), ..
                        } => match x {
                            Keycode::Left | Keycode::Q => {
                                players[0].handle_release(Direction::Left)
                            }
                            Keycode::Right | Keycode::D => {
                                players[0].handle_release(Direction::Right)
                            }
                            Keycode::Up | Keycode::Z => players[0].handle_release(Direction::Up),
                            Keycode::Down | Keycode::S => {
                                players[0].handle_release(Direction::Down)
                            }
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
                        ev => {
                            egui_sdl2_gl::input_to_egui(ev, egui_input_state);
                            // let take_focus = if let Event::MouseMotion { .. } = ev {
                            //     false
                            // } else {
                            //     true
                            // };
                            // let mut i = self.windows.len() - 1;
                            // loop {
                            //     {
                            //         let w = &mut self.windows[i];
                            //         if w.is_hidden() || !w.handle_event(&mut self.widgets, &ev) {
                            //             if i > 0 {
                            //                 i -= 1;
                            //                 continue;
                            //             }
                            //             break;
                            //         }
                            //     }
                            //     if take_focus {
                            //         if i != self.windows.len() - 1 {
                            //             let tmp = self.windows.remove(i);
                            //             self.windows.push(tmp);
                            //         }
                            //     }
                            //     break;
                            // }
                        }
                    }
                }
            }
        }
        true
    }

    pub fn show_death_screen(&mut self, textures: &Textures<'_>) {
        self.menu.set_death(textures);
        self.display_menu = true;
    }

    pub fn debug_draw(&mut self, system: &mut System, player: &Player, elapsed: u32) {
        if self.debug {
            self.fps_str = if elapsed < crate::FRAME_DELAY {
                "FPS: 60.0".to_owned()
            } else {
                format!("FPS: {:.2}", ONE_SECOND as f64 / elapsed as f64)
            };
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
        }
    }

    pub fn draw_rewards(&mut self, system: &mut System, rewards: &[Reward], player: &Player) {
        if self.need_sort_rewards {
            self.closest_reward = None;
            for (i, reward) in rewards.iter().enumerate() {
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
            self.reward_text.draw(
                system,
                reward.x() + (reward.width() as i64) / 2 - (self.reward_text.width as i64) / 2,
                reward.y() - 2 - self.reward_text.height as i64,
            );
        }
    }

    pub fn rumble(&mut self, strength: u16, duration_ms: u32) {
        if let Some(ref mut controller) = self.controller {
            if let Err(e) = controller
                .controller
                .set_rumble(strength, strength, duration_ms)
            {
                eprintln!("cannot set rumble: {:?}", e);
            }
        }
    }

    pub fn draw(&mut self, system: &mut System) {
        if self.display_menu {
            self.menu.draw(system);
        }
    }
}
