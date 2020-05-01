extern crate sdl2;

use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::time::{Duration, Instant};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod character;
mod enemy;
mod hud;
mod map;
mod player;
mod texture_handler;

use enemy::Enemy;
use hud::HUD;
use map::Map;
use player::Player;
use texture_handler::TextureHandler;

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 600;
pub const MAP_SIZE: u32 = 1_000;
pub const FRAME_DELAY: u32 = 1_000_000_000u32 / 60;

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[repr(usize)]
pub enum Direction {
    Front = 0,
    Left = 1,
    Right = 2,
    Back = 3,
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct Action {
    direction: Direction,
    secondary: Option<Direction>,
    movement: Option<u64>,
}

impl Action {
    /// Returns `(x, y, width, height)`.
    pub fn get_current(&self, is_running: bool, textures: &TextureHandler<'_>) -> (i32, i32, i32, i32) {
        if let Some(ref pos) = self.movement {
            let (info, nb_animations) = &textures.actions_moving[self.direction as usize];
            let pos = if is_running {
                (pos % 30) as i32 / (30 / nb_animations)
            } else {
                (pos % 60) as i32 / (60 / nb_animations)
            };
            (pos * info.incr_to_next + info.x, info.y, info.width() as i32, info.height() as i32)
        } else {
            let info = &textures.actions_standing[self.direction as usize];
            (info.x, info.y, info.width() as i32, info.height() as i32)
        }
    }
}

pub fn main() {
    let mut hasher = DefaultHasher::new();
    "hello!".hash(&mut hasher);
    let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let _sdl_img_context = image::init(image::InitFlag::PNG).expect("failed to init SDL image");
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
    let mut enemy = Enemy::new(&texture_creator, 50, 20);
    let hud = HUD::new(&texture_creator);
    let mut screen = Rect::new(
        player.character.x - WIDTH / 2,
        player.character.y - HEIGHT / 2,
        WIDTH as u32,
        HEIGHT as u32,
    );

    let mut loop_timer = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(x), ..
                } => match x {
                    Keycode::Escape => break 'running,
                    Keycode::Left => player.handle_move(Direction::Left),
                    Keycode::Right => player.handle_move(Direction::Right),
                    Keycode::Up => player.handle_move(Direction::Back),
                    Keycode::Down => player.handle_move(Direction::Front),
                    Keycode::LShift => {
                        player.is_run_pressed = true;
                        player.is_running = player.character.action.movement.is_some();
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(x), ..
                } => {
                    match x {
                        // Not complete: if a second direction is pressed, it should then go to this
                        // direction. :)
                        Keycode::Left => player.handle_release(Direction::Left),
                        Keycode::Right => player.handle_release(Direction::Right),
                        Keycode::Up => player.handle_release(Direction::Back),
                        Keycode::Down => player.handle_release(Direction::Front),
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
        // For now, the screen follows the player.
        screen.x = player.character.x - WIDTH / 2;
        screen.y = player.character.y - HEIGHT / 2;
        map.draw(&mut canvas, &screen);
        enemy.draw(&mut canvas, &screen);
        player.draw(&mut canvas, &screen);
        hud.draw(&player, &mut canvas);

        let elapsed_time = loop_timer.elapsed();
        if elapsed_time.as_nanos() < FRAME_DELAY as u128 {
            ::std::thread::sleep(Duration::new(0, FRAME_DELAY - elapsed_time.as_nanos() as u32));
        }
        loop_timer = Instant::now();
    }
}
