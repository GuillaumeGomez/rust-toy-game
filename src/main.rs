use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sdl2::image::{self, LoadSurface};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::ttf;
use sdl2::video::Window;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

#[macro_use]
mod utils;

mod animation;
mod character;
mod debug_display;
mod enemy;
mod env;
mod font_handler;
mod health_bar;
mod hud;
mod map;
mod menu;
mod player;
mod player_stats;
mod reward;
mod stat;
mod status;
mod system;
mod texture_handler;
mod texture_holder;
mod traits;
mod weapon;
mod widgets;
mod window;

use character::CharacterKind;
use enemy::Enemy;
use env::Env;
use health_bar::HealthBar;
use hud::HUD;
use map::Map;
use player::Player;
use reward::Reward;
use system::System;
use texture_holder::TextureHolder;

pub use traits::*;

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 600;
pub const MAP_SIZE: u32 = 1_000;
// in micro-seconds
pub const ONE_SECOND: u64 = 1_000_000;
pub const FPS: u64 = 60;
pub const FRAME_DELAY: u64 = ONE_SECOND / FPS;
pub const MAX_DISTANCE_DETECTION: i32 = 200;
pub const MAX_DISTANCE_PURSUIT: i32 = 300;
pub const MAP_CASE_SIZE: i64 = 8;
/// You need 8 pixels to have 1 "grid case" and you need 4 "grid cases" to have a meter.
pub const PIXELS_TO_METERS: i64 = 8 * 4;
/// Just an alias to `PIXELS_TO_METERS`, to make usage more clear in the code.
pub const ONE_METER: i64 = PIXELS_TO_METERS;
pub const MAX_DISTANCE_WANDERING: i32 = ONE_METER as i32 * 15;

/// Just for code clarity.
pub type Id = usize;

macro_rules! load_font {
    ($ttf_context:expr, $size:expr) => {{
        $ttf_context
            .load_font("resources/kreon-regular.ttf", $size)
            .expect("failed to load `resources/kreon-regular.ttf`")
    }};
}

pub fn draw_in_good_order(
    system: &mut System,
    debug: bool,
    players: &mut Vec<Player>,
    enemies: &mut Vec<Enemy>,
    dead_enemies: &mut Vec<Enemy>,
) {
    players.sort_unstable_by(|c1, c2| c1.y().cmp(&c2.y()));
    enemies.sort_unstable_by(|c1, c2| c1.y().cmp(&c2.y()));
    dead_enemies.sort_unstable_by(|c1, c2| c1.y().cmp(&c2.y()));

    let mut player_iter = players.iter_mut().peekable();
    let mut enemy_iter = enemies.iter_mut().peekable();
    let mut dead_enemy_iter = dead_enemies.iter_mut().peekable();

    while player_iter.peek().is_some()
        || enemy_iter.peek().is_some()
        || dead_enemy_iter.peek().is_some()
    {
        if let Some(ref player) = player_iter.peek() {
            let y = player.y();
            if y < enemy_iter.peek().map(|x| x.y()).unwrap_or(y + 1)
                && y < dead_enemy_iter.peek().map(|x| x.y()).unwrap_or(y + 1)
            {
                player_iter.next().unwrap().draw(system, debug);
                continue;
            }
        }
        if let Some(ref enemy) = enemy_iter.peek() {
            let y = enemy.y();
            if y < dead_enemy_iter.peek().map(|x| x.y()).unwrap_or(y + 1) {
                enemy_iter.next().unwrap().draw(system, debug);
                continue;
            }
        }
        dead_enemy_iter.next().unwrap().draw(system, false);
    }
}

pub fn main() {
    // This is the seed used to generate the same world based on its name.
    let mut hasher = DefaultHasher::new();
    // TODO: take this hash from the world name/save file.
    "hello!".hash(&mut hasher);
    let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let _sdl_img_context = image::init(image::InitFlag::PNG).expect("failed to init SDL image");
    let game_controller_subsystem = sdl_context
        .game_controller()
        .expect("failed to init game controller subsystem");

    let ttf_context = ttf::init().expect("failed to init SDL TTF");
    let video_subsystem = sdl_context.video().expect("failed to get video context");

    let (mut width, mut height) = match video_subsystem.current_display_mode(0) {
        Ok(info) => {
            let h = info.h as u32 / 2;
            (h * 4 / 3, h)
        }
        Err(_) => (WIDTH as u32, HEIGHT as u32),
    };
    if width < WIDTH as u32 || height < HEIGHT as u32 {
        width = WIDTH as u32;
        height = HEIGHT as u32;
    }

    let window = video_subsystem
        .window("toy game", width, height)
        .position_centered()
        .resizable()
        .build()
        .expect("failed to build window");

    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("failed to build window's canvas");
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    // Very important in case the window is resized and isn't 4/3 ratio anymore!
    canvas.set_logical_size(WIDTH as _, HEIGHT as _).expect("failed to set logical size");
    let texture_creator = canvas.texture_creator();
    let health_bar = HealthBar::new(&texture_creator, 30, 5);
    let mut system = System::new(canvas, WIDTH as _, HEIGHT as _, &health_bar);

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let map = Map::new(
        &texture_creator,
        &mut rng,
        MAP_SIZE as i64 * MAP_CASE_SIZE / -2,
        MAP_SIZE as i64 * MAP_CASE_SIZE / -2,
    );

    let font_10 = load_font!(ttf_context, 10);
    let font_12 = load_font!(ttf_context, 12);
    let font_14 = load_font!(ttf_context, 14);
    let font_16 = load_font!(ttf_context, 16);

    system.create_new_font_map(&texture_creator, &font_14, 14, Color::RGB(255, 0, 0));
    system.create_new_font_map(&texture_creator, &font_12, 12, Color::RGB(255, 255, 255));
    system.create_new_font_map(&texture_creator, &font_16, 16, Color::RGB(255, 255, 255));
    system.create_new_font_map(&texture_creator, &font_16, 16, Color::RGB(74, 138, 221));

    let (player_texture, player_surface) = player::get_player(&texture_creator);
    let mut enemy_surface = Surface::from_file("resources/skeleton.png")
        .expect("failed to load `resources/skeleton.png`");
    if enemy_surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
        enemy_surface = enemy_surface
            .convert_format(PixelFormatEnum::RGBA8888)
            .expect("failed to convert surface to RGBA8888");
    }
    let enemy_texture = texture_creator
        .create_texture_from_surface(&enemy_surface)
        .expect("failed to build texture from surface");
    let mut forced_enemy_surface = Surface::new(24 * 3, 24 * 4, enemy_surface.pixel_format_enum())
        .expect("failed to create new surface for resize");
    let rect = forced_enemy_surface.rect();
    enemy_surface
        .blit(None, &mut forced_enemy_surface, rect)
        .expect("failed to resize surface...");

    // TODO: maybe move that in `Env`?
    let mut textures = HashMap::new();
    textures.insert(
        "reward",
        TextureHolder::from_image(&texture_creator, "resources/bag.png").with_max_size(24),
    );
    textures.insert(
        "reward-text",
        TextureHolder::from_text(
            &texture_creator,
            &font_10,
            Color::RGB(0, 0, 0),
            None,
            "Press ENTER",
        ),
    );
    animation::create_death_animation_texture(&mut textures, &texture_creator);
    animation::create_level_up_animation_texture(&mut textures, &texture_creator);

    let hud = HUD::new(&texture_creator);
    Env::init_textures(&mut textures, &texture_creator, WIDTH as u32, HEIGHT as u32);
    let mut env = Env::new(
        &game_controller_subsystem,
        &texture_creator,
        &textures,
        WIDTH as u32,
        HEIGHT as u32,
    );
    let mut rewards = Vec::new();

    let mut update_elapsed = 0;
    let mut loop_timer = Instant::now();

    let mut players = vec![Player::new(
        &texture_creator,
        &player_texture,
        &player_surface,
        0,
        0,
        1,
        Some(Default::default()),
        Some(&mut env),
    )];
    let mut enemies = vec![
        Enemy::new(
            &texture_creator,
            &textures,
            &enemy_texture,
            &forced_enemy_surface,
            0,
            40,
            2,
            CharacterKind::Enemy,
            enemy_surface.width() / 3,
            enemy_surface.height() / 4,
        ),
        Enemy::new(
            &texture_creator,
            &textures,
            &enemy_texture,
            &forced_enemy_surface,
            40,
            0,
            3,
            CharacterKind::Enemy,
            enemy_surface.width() / 3,
            enemy_surface.height() / 4,
        ),
    ];

    let mut dead_enemies: Vec<Enemy> = Vec::new();

    loop {
        if !env.handle_events(&mut event_pump, &mut players, &mut rewards, &textures) {
            break;
        }

        if !env.display_menu {
            for it in (0..dead_enemies.len()).rev() {
                dead_enemies[it].update(update_elapsed, 0, 0);
                if dead_enemies[it].should_be_removed() {
                    // TODO: in here, give XP to the playerS depending on how much
                    //       damage they did to the monster and with a bonus/malus based
                    //       on the level difference.
                    if let Some(reward) = dead_enemies[it].get_reward() {
                        let texture = &textures["reward"];
                        let width = texture.width as i32;
                        let height = texture.height as i32;
                        rewards.push(Reward::new(
                            texture,
                            dead_enemies[it].x()
                                + ((dead_enemies[it].width() as i32 / 2) - width / 2) as i64,
                            dead_enemies[it].y()
                                + ((dead_enemies[it].height() as i32 / 2) - height / 2) as i64,
                            reward,
                        ));
                        env.need_sort_rewards = true;
                    }
                    dead_enemies.remove(it);
                }
            }

            if env.is_attack_pressed && !players[0].is_attacking() {
                players[0].attack();
            }
            let len = players.len();
            for i in 0..len {
                let (x, y) = players[i].apply_move(&map, update_elapsed, &players, &enemies);
                if i == 0 && (x != 0 || y != 0) {
                    env.need_sort_rewards = true;
                }
                if let Some(ref stats) = players[i].stats {
                    stats.borrow_mut().total_walked += ::std::cmp::max(x.abs(), y.abs()) as u64;
                }
                players[i].update(
                    update_elapsed,
                    x,
                    y,
                    if i == 0 { Some(&mut env) } else { None },
                );
                if players[i].is_attacking() {
                    let id = players[i].id;
                    let dir = players[i].get_direction();
                    let mut xp_to_add = 0;
                    if let Some(ref weapon) = players[i].weapon {
                        let mut matrix = None;
                        // TODO: for now, players can only attack NPCs
                        for it in (0..enemies.len()).rev() {
                            let attack =
                                enemies[it].check_intersection(id, dir, weapon, &mut matrix);
                            if attack > 0 {
                                if i == 0 {
                                    env.rumble(u16::MAX / 13, 250);
                                }
                                let is_dead = enemies[it].is_dead();
                                if let Some(ref stats) = players[i].stats {
                                    let mut stats = stats.borrow_mut();
                                    if attack > 0 {
                                        stats.total_damages.total_inflicted_damages +=
                                            attack as u64;
                                        if is_dead {
                                            stats.total_damages.total_kills += 1;
                                            xp_to_add += enemies[it].xp;
                                        }
                                    } else {
                                        stats.total_damages.total_healed += (attack * -1) as u64;
                                    }
                                    let enemy_stats = stats
                                        .enemies
                                        .entry(enemies[it].kind)
                                        .or_insert_with(Default::default);
                                    if attack > 0 {
                                        enemy_stats.total_inflicted_damages += attack as u64;
                                        if is_dead {
                                            enemy_stats.total_kills += 1;
                                        }
                                    } else {
                                        enemy_stats.total_healed += (attack * -1) as u64;
                                    }
                                }
                                if is_dead {
                                    dead_enemies.push(enemies.remove(it));
                                }
                            }
                        }
                    }
                    if xp_to_add > 0 {
                        players[i].increase_xp(
                            xp_to_add,
                            &textures,
                            if i == 0 { Some(&mut env) } else { None },
                        );
                    }
                }
            }
            let len = enemies.len();
            for i in 0..len {
                let (x, y) = enemies[i].apply_move(&map, update_elapsed, &players, &enemies);
                enemies[i].update(update_elapsed, x, y);
                if enemies[i].is_attacking() {
                    let id = enemies[i].id;
                    let dir = enemies[i].get_direction();
                    if let Some(ref weapon) = enemies[i].weapon {
                        let mut matrix = None;
                        // TODO: for now, NPCs can only attack players
                        for (pos, player) in players.iter_mut().enumerate() {
                            if player.check_intersection(id, dir, weapon, &mut matrix) > 0
                                && pos == 0
                            {
                                env.rumble(u16::MAX / 10, 250);
                            }
                        }
                    }
                }
            }
            if players[0].is_dead() {
                env.show_death_screen(&textures);
            }
        }

        system.clear();
        // TODO: instead of having draw methods on each drawable objects, maybe create a Screen
        // type which will get position, size and texture and perform the checks itself? Might be
        // a bit complicated in case an object contains objects to draw though... It could be
        // overcome by adding a methods "get_drawable_children" though.
        //
        // For now, the screen follows the player.
        system.set_screen_position(&players[0]);
        map.draw(&mut system);
        // TODO: put this whole thing somewhere else
        env.draw_rewards(&mut system, &rewards, &players[0], &textures);
        draw_in_good_order(
            &mut system,
            env.debug,
            &mut players,
            &mut enemies,
            &mut dead_enemies,
        );
        map.draw_layer(&mut system);
        hud.draw(&players[0], &mut system);

        env.draw(&mut system);

        let elapsed_time = loop_timer.elapsed();

        let micro_elapsed = elapsed_time.as_micros() as u64;
        update_elapsed = if micro_elapsed < FRAME_DELAY {
            let tmp = FRAME_DELAY - micro_elapsed;
            ::std::thread::sleep(Duration::from_micros(tmp));
            tmp
        } else {
            micro_elapsed
        } as u64;
        // TODO: use `update_elapsed` instead of `loop_timer` for the FPS count!
        env.debug_draw(&mut system, &players[0], micro_elapsed);
        loop_timer = Instant::now();
    }
}
