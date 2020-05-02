extern crate sdl2;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf;
use sdl2::video::Window;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

mod character;
mod debug_display;
mod enemy;
mod hud;
mod map;
mod player;
mod texture_handler;
mod utils;

use character::Direction;
use debug_display::DebugDisplay;
use enemy::Enemy;
use hud::HUD;
use map::Map;
use player::Player;

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 600;
pub const MAP_SIZE: u32 = 1_000;
pub const FRAME_DELAY: u32 = 1_000_000_000 / 60;
pub const MAX_DISTANCE_DETECTION: i32 = 200;
pub const MAX_DISTANCE_PURSUIT: i32 = 300;
pub const MAX_DISTANCE_WANDERING: i32 = 300;

const FPS_REFRESH: u32 = 5;

pub trait GetPos {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
}

impl GetPos for (i32, i32) {
    fn x(&self) -> i32 {
        self.0
    }
    fn y(&self) -> i32 {
        self.1
    }
}

pub trait GetDimension {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub fn main() {
    // This is the seed used to generate the same world based on its name.
    let mut hasher = DefaultHasher::new();
    "hello!".hash(&mut hasher);
    let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let _sdl_img_context = image::init(image::InitFlag::PNG).expect("failed to init SDL image");

    let ttf_context = ttf::init().expect("failed to init SDL TTF");

    let video_subsystem = sdl_context.video().expect("failed to get video context");

    let window = video_subsystem
        .window("toy game", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .expect("failed to build window");

    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("failed to build window's canvas");
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let map = Map::new(&texture_creator, &mut rng);
    let mut player = Player::new(&texture_creator, 0, 0);
    let mut enemy = Enemy::new(&texture_creator, -40, -40);
    let hud = HUD::new(&texture_creator);
    let mut screen = Rect::new(
        player.character.x - WIDTH / 2,
        player.character.y - HEIGHT / 2,
        WIDTH as u32,
        HEIGHT as u32,
    );
    let font = ttf_context
        .load_font("resources/kreon-regular.ttf", 16)
        .expect("failed to load `resources/kreon-regular.ttf`");
    let debug_display = DebugDisplay::new(&font, &texture_creator, 16);
    let mut debug = None;
    let mut fps_str = String::new();

    let mut loop_timer = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(x), ..
                } => match x {
                    Keycode::Escape => break 'running,
                    Keycode::Left | Keycode::Q => player.handle_move(Direction::Left),
                    Keycode::Right | Keycode::D => player.handle_move(Direction::Right),
                    Keycode::Up | Keycode::Z => player.handle_move(Direction::Back),
                    Keycode::Down | Keycode::S => player.handle_move(Direction::Front),
                    Keycode::LShift => {
                        player.is_run_pressed = true;
                        player.is_running = player.character.action.movement.is_some();
                    }
                    Keycode::F3 => {
                        if debug.is_some() {
                            debug = None;
                        } else {
                            debug = Some(FPS_REFRESH - 1);
                        }
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(x), ..
                } => {
                    match x {
                        // Not complete: if a second direction is pressed, it should then go to this
                        // direction. :)
                        Keycode::Left | Keycode::Q => player.handle_release(Direction::Left),
                        Keycode::Right | Keycode::D => player.handle_release(Direction::Right),
                        Keycode::Up | Keycode::Z => player.handle_release(Direction::Back),
                        Keycode::Down | Keycode::S => player.handle_release(Direction::Front),
                        Keycode::LShift => {
                            player.is_run_pressed = false;
                            player.is_running = false;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        canvas.present();
        canvas.clear();

        player.apply_move(&map);
        enemy.update(&player, &map);
        // For now, the screen follows the player.
        screen.x = player.character.x - WIDTH / 2;
        screen.y = player.character.y - HEIGHT / 2;
        map.draw(&mut canvas, &screen);
        enemy.draw(&mut canvas, &screen);
        player.draw(&mut canvas, &screen);
        hud.draw(&player, &mut canvas);


        let elapsed_time = loop_timer.elapsed();

        if elapsed_time.as_nanos() < FRAME_DELAY as u128 {
            ::std::thread::sleep(Duration::new(
                0,
                FRAME_DELAY - elapsed_time.as_nanos() as u32,
            ));
        }
        if let Some(ref mut debug) = debug {
            *debug += 1;
            if *debug >= FPS_REFRESH {
                let elapsed_time = loop_timer.elapsed();
                fps_str = format!(
                    "FPS: {:.2}",
                    1_000_000_000f64 / elapsed_time.as_nanos() as f64
                );
                *debug = 0;
            }
            debug_display.draw(&mut canvas, &format!("{}\nposition: ({}, {})", fps_str, player.x(), player.y()));
        }
        loop_timer = Instant::now();
    }
}
