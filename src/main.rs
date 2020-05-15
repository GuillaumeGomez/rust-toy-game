extern crate sdl2;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sdl2::image;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::ttf;
use sdl2::video::Window;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

macro_rules! return_if_none {
    ($x:expr) => {{
        match $x {
            Some(x) => x,
            None => return,
        }
    }};
}

mod character;
mod debug_display;
mod enemy;
mod env;
mod health_bar;
mod hud;
mod map;
mod menu;
mod player;
mod stat;
mod status;
mod system;
mod texture_handler;
mod utils;
mod weapon;

use enemy::Enemy;
use env::Env;
use health_bar::HealthBar;
use hud::HUD;
use map::Map;
use player::Player;
use system::System;

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 600;
pub const MAP_SIZE: u32 = 1_000;
// in micro-seconds
pub const ONE_SECOND: u64 = 1_000_000;
pub const FPS: u64 = 60;
pub const FRAME_DELAY: u64 = ONE_SECOND / FPS;
pub const MAX_DISTANCE_DETECTION: i32 = 200;
pub const MAX_DISTANCE_PURSUIT: i32 = 300;
pub const MAX_DISTANCE_WANDERING: i32 = 300;
pub const MAP_CASE_SIZE: i64 = 8;

const FPS_REFRESH: u32 = 5;

/// Just for code clarity.
pub type Id = usize;

pub trait GetPos {
    fn x(&self) -> i64;
    fn y(&self) -> i64;
}

impl GetPos for (i64, i64) {
    fn x(&self) -> i64 {
        self.0
    }
    fn y(&self) -> i64 {
        self.1
    }
}

pub trait GetDimension {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

macro_rules! load_font {
    ($ttf_context:expr, $size:expr) => {{
        $ttf_context
            .load_font("resources/kreon-regular.ttf", $size)
            .expect("failed to load `resources/kreon-regular.ttf`")
    }};
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
    let health_bar = HealthBar::new(&texture_creator, 30, 5);
    let mut system = System::new(canvas, WIDTH as u32, HEIGHT as u32, &health_bar);

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let map = Map::new(
        &texture_creator,
        &mut rng,
        MAP_SIZE as i64 * MAP_CASE_SIZE / -2,
        MAP_SIZE as i64 * MAP_CASE_SIZE / -2,
    );
    let mut players = vec![Player::new(&texture_creator, 0, 0, 1)];
    let mut enemies = vec![Enemy::new(&texture_creator, -40, -40, 2)];

    let font_14 = load_font!(ttf_context, 14);
    let font_16 = load_font!(ttf_context, 16);

    let hud = HUD::new(&texture_creator);
    let mut env = Env::new(&texture_creator, &font_16, WIDTH as u32, HEIGHT as u32);

    let mut update_elapsed = 0;
    let mut loop_timer = Instant::now();

    loop {
        if !env.handle_events(&mut event_pump, &mut players) {
            break;
        }

        if !env.display_menu {
            if env.is_attack_pressed && !players[0].is_attacking() {
                players[0].attack();
            }
            let len = players.len();
            for i in 0..len {
                let (x, y) = players[i].apply_move(&map, update_elapsed, &players, &enemies);
                players[i].update(update_elapsed, x, y);
                if players[i].is_attacking() {
                    let id = players[i].id;
                    if let Some(ref weapon) = players[i].weapon {
                        let mut matrix = None;
                        // TODO: for now, players can only attack NPCs
                        for enemy in enemies.iter_mut() {
                            enemy.check_intersection(
                                id,
                                weapon,
                                &mut matrix,
                                &font_14,
                                &texture_creator,
                            );
                        }
                    }
                }
            }
            let len = enemies.len();
            for i in 0..len {
                let (x, y) = enemies[i].apply_move(&map, update_elapsed, &players, &enemies);
                enemies[i].update(update_elapsed, x, y);
                if enemies[i].is_attacking() {
                    let id = enemies[i].id;
                    if let Some(ref weapon) = enemies[i].weapon {
                        let mut matrix = None;
                        // TODO: for now, NPCs can only attack players
                        for player in players.iter_mut() {
                            player.check_intersection(
                                id,
                                weapon,
                                &mut matrix,
                                &font_14,
                                &texture_creator,
                            );
                        }
                    }
                }
            }
        }

        system.clear();
        // TODO: instead of having draw methods on each drawable objects, maybe create a Screen
        // type which will get position, size and texture and perform the checks itself? Might be
        // a bit complicated in case an object contains objects to draw though... It could be overcome
        // by adding a methods "get_drawable_children" though.
        //
        // For now, the screen follows the player.
        system.set_screen_position(&players[0]);
        map.draw(&mut system);
        for player in players.iter_mut() {
            player.draw(&mut system);
        }
        for enemy in enemies.iter_mut() {
            enemy.draw(&mut system);
        }
        hud.draw(&players[0], &mut system);

        if env.display_menu {
            env.menu.draw(&mut system);
        }

        let elapsed_time = loop_timer.elapsed();

        let micro_elapsed = elapsed_time.as_micros() as u64;
        update_elapsed = if micro_elapsed < FRAME_DELAY {
            let tmp = FRAME_DELAY - micro_elapsed;
            ::std::thread::sleep(Duration::from_micros(tmp));
            tmp
        } else {
            micro_elapsed
        } as u64;
        env.debug_draw(&mut system, &players[0], &loop_timer);
        loop_timer = Instant::now();
    }
}
