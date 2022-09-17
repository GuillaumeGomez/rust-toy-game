pub use egui_sdl2_gl::{egui, gl, sdl2};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use sdl2::image::{self, LoadSurface};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf;
use sdl2::video::{GLProfile, Window, WindowContext};

use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

#[macro_use]
mod utils;

mod animation;
mod character;
mod debug_display;
mod enemies;
mod enemy;
mod env;
mod environment;
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
mod weapons;

use environment::Building;
use enemies::{Bat, Skeleton};
use enemy::Enemy;
use env::Env;
use health_bar::HealthBar;
use hud::HUD;
use map::Map;
use player::Player;
use reward::Reward;
use system::System;
use texture_holder::{TextureHolder, Textures};

pub use traits::*;

pub const WIDTH: i32 = 800;
pub const HEIGHT: i32 = 600;
pub const MAP_SIZE: u32 = 1_000;
pub const MAP_CASE_SIZE: i32 = 8;
pub const MAP_SIZE_WITH_CASE: u32 = MAP_CASE_SIZE as u32 * MAP_SIZE;
// in micro-seconds
pub const ONE_SECOND: u32 = 1_000_000;
pub const FPS: u32 = 60;
pub const FRAME_DELAY: u32 = ONE_SECOND / FPS;
pub const MAX_DISTANCE_DETECTION: i32 = 200;
pub const MAX_DISTANCE_PURSUIT: i32 = 300;
/// You need 8 pixels to have 1 "grid case" and you need 4 "grid cases" to have a meter.
pub const PIXELS_TO_METERS: i32 = 8 * 4;
/// Just an alias to `PIXELS_TO_METERS`, to make usage more clear in the code.
pub const ONE_METER: i32 = PIXELS_TO_METERS;
pub const MAX_DISTANCE_WANDERING: i32 = ONE_METER as i32 * 15;
pub const FLOAT_COMPARISON_PRECISION: f32 = 0.001;

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
    enemies: &mut Vec<Box<dyn Enemy>>,
    dead_enemies: &mut Vec<Box<dyn Enemy>>,
    buildings: &mut Vec<Building>,
    sort: bool,
) {
    if sort {
        players.sort_unstable_by(|c1, c2| c1.y().partial_cmp(&c2.y()).unwrap_or(Ordering::Equal));
        enemies.sort_unstable_by(|c1, c2| c1.y().partial_cmp(&c2.y()).unwrap_or(Ordering::Equal));
        dead_enemies
            .sort_unstable_by(|c1, c2| c1.y().partial_cmp(&c2.y()).unwrap_or(Ordering::Equal));
    }

    let mut player_iter = players.iter_mut().peekable();
    let mut enemy_iter = enemies.iter_mut().peekable();
    let mut dead_enemy_iter = dead_enemies.iter_mut().peekable();
    let mut building_iter = buildings.iter_mut().peekable();

    let player_y = player_iter.peek().map(|p| p.y());
    let enemy_y = enemy_iter.peek().map(|p| p.y());
    let dead_enemy_y = dead_enemy_iter.peek().map(|p| p.y());
    let building_y = building_iter.peek().map(|p| p.y());

    let mut poses = [(player_y, 0), (enemy_y, 1), (dead_enemy_y, 2), (building_y, 3)];

    macro_rules! handle_it {
        ($name:ident, $system:ident, $debug:ident, $pos:ident) => {{
            $name.next().unwrap().draw($system, $debug);
            $pos.0 = $name.peek().map(|p| p.y());
        }}
    }

    while poses.iter().any(|x| x.0.is_some()) {
        poses.sort_unstable_by(|c1, c2| c1.0.partial_cmp(&c2.0).unwrap_or(Ordering::Equal));
        let pos = poses.iter_mut().filter(|x| x.0.is_some()).next().unwrap();

        match pos.1 {
            0 => handle_it!(player_iter, system, debug, pos),
            1 => handle_it!(enemy_iter, system, debug, pos),
            2 => handle_it!(dead_enemy_iter, system, debug, pos),
            3 => handle_it!(building_iter, system, debug, pos),
            _ => unreachable!(),
        }
    }
}

fn make_enemies<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    textures: &mut Textures<'a>,
) -> (Vec<Box<dyn Enemy>>, Vec<Box<dyn Enemy>>) {
    let mut skeleton_surface = Surface::from_file("resources/skeleton.png")
        .expect("failed to load `resources/skeleton.png`");
    if skeleton_surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
        skeleton_surface = skeleton_surface
            .convert_format(PixelFormatEnum::RGBA8888)
            .expect("failed to convert surface to RGBA8888");
    }
    let skeleton_texture = texture_creator
        .create_texture_from_surface(&skeleton_surface)
        .expect("failed to build texture from surface");
    let mut forced_skeleton_surface =
        Surface::new(24 * 3, 24 * 4, skeleton_surface.pixel_format_enum())
            .expect("failed to create new surface for resize");

    let rect = forced_skeleton_surface.rect();
    skeleton_surface
        .blit(None, &mut forced_skeleton_surface, rect)
        .expect("failed to resize surface...");

    textures.add_named_texture(
        "skeleton",
        TextureHolder {
            texture: skeleton_texture,
            width: skeleton_surface.width(),
            height: skeleton_surface.height(),
        },
    );
    let width = skeleton_surface.width();
    let height = skeleton_surface.height();
    textures.add_surface("skeleton", skeleton_surface);

    Bat::init_textures(texture_creator, textures);

    let enemies: Vec<Box<dyn Enemy>> = vec![
        Box::new(Skeleton::new(
            texture_creator,
            textures,
            -20.,
            40.,
            2,
            width / 3,
            height / 4,
        )),
        Box::new(Skeleton::new(
            texture_creator,
            textures,
            -60.,
            0.,
            3,
            width / 3,
            height / 4,
        )),
        Box::new(Bat::new(texture_creator, textures, 40., 40., 4)),
    ];

    let dead_enemies = Vec::new();

    (enemies, dead_enemies)
}

fn init_textures<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    textures: &mut Textures<'a>,
) {
    Player::init_textures(&texture_creator, textures);
    crate::weapons::Sword::init_textures(&texture_creator, textures);
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

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);

    // OpenGL 3.2 is the minimum that we will support.
    gl_attr.set_context_version(3, 2);

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
        .opengl()
        .build()
        .expect("failed to build window");

    let mut canvas: Canvas<Window> = window
        .into_canvas()
        // .present_vsync()
        .accelerated()
        .build()
        .expect("failed to build window's canvas");
    canvas.set_draw_color(Color::BLACK);
    // Very important in case the window is resized and isn't 4/3 ratio anymore!
    canvas
        .set_logical_size(WIDTH as _, HEIGHT as _)
        .expect("failed to set logical size");
    let texture_creator = canvas.texture_creator();
    let health_bar = HealthBar::new(&texture_creator, 30, 5);

    // let native_pixels_per_point = 150f32 / video_subsystem.display_dpi(0).unwrap().0;
    let (mut painter, mut egui_input_state) =
        egui_sdl2_gl::with_sdl2(canvas.window(), egui_sdl2_gl::ShaderVersion::Adaptive, egui_sdl2_gl::DpiScaling::Custom(1.5));
    // let mut painter = egui_sdl2_gl::Painter::new(&video_subsystem, WIDTH as _, HEIGHT as _);
    let mut egui_ctx = egui_sdl2_gl::egui::CtxRef::default();
    // let mut egui_input_state = egui_sdl2_gl::EguiInputState::new(egui::RawInput {
    //     screen_rect: Some(egui::Rect::from_min_size(
    //         egui::Pos2::new(0f32, 0f32),
    //         egui::vec2(width as f32, height as f32) / native_pixels_per_point,
    //     )),
    //     pixels_per_point: Some(native_pixels_per_point),
    //     ..Default::default()
    // });

    let font_10 = load_font!(ttf_context, 10);
    let font_12 = load_font!(ttf_context, 12);
    let font_14 = load_font!(ttf_context, 14);
    let font_16 = load_font!(ttf_context, 16);

    let mut system = System::new(
        canvas,
        WIDTH as _,
        HEIGHT as _,
        &health_bar,
        Textures::new(),
    );

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let map = Map::new(
        &texture_creator,
        &mut rng,
        (MAP_SIZE as i32 * MAP_CASE_SIZE / -2) as _,
        (MAP_SIZE as i32 * MAP_CASE_SIZE / -2) as _,
    );

    init_textures(&texture_creator, &mut system.textures);

    system.create_new_font_map(&texture_creator, &font_14, 14, Color::RGB(255, 0, 0));
    system.create_new_font_map(&texture_creator, &font_12, 12, Color::RGB(255, 255, 255));
    system.create_new_font_map(&texture_creator, &font_16, 16, Color::RGB(255, 255, 255));
    system.create_new_font_map(&texture_creator, &font_16, 16, Color::RGB(74, 138, 221));

    let reward_id = system.textures.add_texture(
        TextureHolder::from_image(&texture_creator, "resources/bag.png").with_max_size(24),
    );
    system.textures.add_named_texture(
        "reward-text",
        TextureHolder::from_text(
            &texture_creator,
            &font_10,
            Color::RGB(0, 0, 0),
            None,
            "Press ENTER",
        ),
    );

    animation::create_death_animation_texture(&mut system.textures, &texture_creator);
    animation::create_level_up_animation_texture(&mut system.textures, &texture_creator);

    let hud = HUD::new(&texture_creator);
    Env::init_textures(
        &mut system.textures,
        &texture_creator,
        WIDTH as _,
        HEIGHT as _,
    );
    let mut env = Env::new(
        &game_controller_subsystem,
        &texture_creator,
        &mut system.textures,
        WIDTH as _,
        HEIGHT as _,
    );
    let mut rewards = Vec::new();

    let mut update_elapsed = 0;
    let mut loop_timer = Instant::now();

    let mut players = vec![Player::new(
        &system.textures,
        0.,
        0.,
        1,
        Some(Default::default()),
        Some(&mut env),
    )];
    let mut buildings: Vec<Building> = Vec::new();
    let (mut enemies, mut dead_enemies) = make_enemies(&texture_creator, &mut system.textures);
    let mut sort_update = 0u8;
    let start_time = Instant::now();

    loop {
        egui_input_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_input_state.input.take());
        // egui_input_state.input.pixels_per_point = Some(native_pixels_per_point);

        if !env.handle_events(
            &mut event_pump,
            &mut players,
            &mut rewards,
            &system.textures,
            &mut egui_input_state,
            system.window(),
            &mut painter,
        ) {
            break;
        }

        if !env.display_menu {
            // FIXME: use `.iter_mut().retain()` instead!
            for it in (0..dead_enemies.len()).rev() {
                let to_remove = {
                    dead_enemies[it].update(update_elapsed, 0., 0.);
                    let dead_enemy = &dead_enemies[it];
                    if dead_enemy.character().should_be_removed() {
                        // TODO: in here, give XP to the playerS depending on how much
                        //       damage they did to the monster and with a bonus/malus based
                        //       on the level difference.
                        if let Some(reward) = dead_enemies[it].character().get_reward() {
                            let width = reward_id.width as i32;
                            let height = reward_id.height as i32;
                            rewards.push(Reward::new(
                                reward_id,
                                dead_enemy.x()
                                    + ((dead_enemies[it].width() as i32 / 2) - width / 2) as f32,
                                dead_enemy.y()
                                    + ((dead_enemies[it].height() as i32 / 2) - height / 2) as f32,
                                reward,
                            ));
                            env.need_sort_rewards = true;
                        }
                        true
                    } else {
                        false
                    }
                };
                if to_remove {
                    dead_enemies.remove(it);
                }
            }

            if env.is_attack_pressed && !players[0].is_attacking() {
                players[0].attack();
            }
            let len = players.len();
            for i in 0..len {
                let (x, y) = players[i].apply_move(&map, update_elapsed, &players, &enemies);
                if i == 0 && (x != 0. || y != 0.) {
                    env.need_sort_rewards = true;
                }
                if let Some(ref stats) = players[i].stats {
                    stats.borrow_mut().total_walked += x.abs().max(y.abs()) as u64;
                }
                players[i].update(
                    update_elapsed,
                    x,
                    y,
                    if i == 0 { Some(&mut env) } else { None },
                );
                if players[i].is_attacking() {
                    let player = &players[i];
                    let mut xp_to_add = 0;
                    let mut matrix = None;
                    // TODO: for now, players can only attack NPCs
                    for it in (0..enemies.len()).rev() {
                        let attack = enemies[it].character().check_intersection(
                            player,
                            &mut matrix,
                            &system.textures,
                        );
                        if attack > 0 {
                            enemies[it].character_mut().update_attack_info(
                                player.id,
                                player.weapon.total_time,
                                attack,
                            );
                            if i == 0 {
                                env.rumble(u16::MAX / 13, 250);
                            }
                            let is_dead = enemies[it].character().is_dead();
                            if let Some(ref stats) = players[i].stats {
                                let mut stats = stats.borrow_mut();
                                if attack > 0 {
                                    stats.total_damages.total_inflicted_damages += attack as u64;
                                    if is_dead {
                                        stats.total_damages.total_kills += 1;
                                        xp_to_add += enemies[it].character().xp;
                                    }
                                } else {
                                    stats.total_damages.total_healed += (attack * -1) as u64;
                                }
                                let enemy_stats = stats
                                    .enemies
                                    .entry(enemies[it].character().kind)
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
                    if xp_to_add > 0 {
                        players[i].increase_xp(
                            xp_to_add,
                            &system.textures,
                            if i == 0 { Some(&mut env) } else { None },
                        );
                    }
                }
            }
            let len = enemies.len();
            for i in 0..len {
                let (x, y) = enemies[i].apply_move(&map, update_elapsed, &players, &enemies);
                let enemy = &mut enemies[i];
                enemy.update(update_elapsed, x, y);
                if enemy.character().is_attacking() {
                    let mut matrix = None;
                    // TODO: for now, NPCs can only attack players
                    for (pos, player) in players.iter_mut().enumerate() {
                        let attack = player.check_intersection(
                            enemy.character(),
                            &mut matrix,
                            &system.textures,
                        );
                        if attack != 0 {
                            player.character.update_attack_info(
                                enemy.id(),
                                enemy.character().weapon.total_time,
                                attack,
                            );
                            // If this the current player (not a remote one), then we vibrate
                            // the controller.
                            if pos == 0 {
                                env.rumble(u16::MAX / 10, 250);
                            }
                        }
                    }
                }
            }
            if players[0].is_dead() {
                env.show_death_screen(&system.textures);
            }
        }

        // TODO: instead of having draw methods on each drawable objects, maybe create a Screen
        // type which will get position, size and texture and perform the checks itself? Might be
        // a bit complicated in case an object contains objects to draw though... It could be
        // overcome by adding a methods "get_drawable_children" though.
        //
        // For now, the screen follows the player.
        system.set_screen_position(&players[0]);
        map.draw(&mut system);
        // TODO: put this whole thing somewhere else
        env.draw_rewards(&mut system, &rewards, &players[0]);
        draw_in_good_order(
            &mut system,
            env.debug,
            &mut players,
            &mut enemies,
            &mut dead_enemies,
            &mut buildings,
            sort_update == 10,
        );
        map.draw_layer(&mut system);
        hud.draw(&players[0], &mut system);

        env.draw(&mut system);

        let elapsed_time = loop_timer.elapsed();
        let micro_elapsed = elapsed_time.as_micros() as u32;

        // TODO: use `update_elapsed` instead of `loop_timer` for the FPS count!
        env.debug_draw(&mut system, &players[0], micro_elapsed);

        // Needed to make all SDL opengl calls.
        unsafe { system.canvas.render_flush() };

        if env.character_window.is_displayed {
            egui::Window::new("Character information")
                .collapsible(false)
                .open(&mut env.character_window.is_displayed)
                .show(&egui_ctx, |ui| {
                    let player = &mut players[0].character;

                    ui.vertical_centered_justified(|ui| {
                        ui.label("Player imperio");
                    });
                    ui.separator();

                    egui::Grid::new("character_infos").show(ui, |ui| {
                        ui.label("Level");
                        ui.label(&player.level.to_string());
                        ui.end_row();

                        ui.label("Experience");
                        ui.label(&format!("{} / {}", player.xp, player.xp_to_next_level));
                        ui.end_row();

                        let stats = &player.stats;

                        ui.label("Health");
                        ui.label(&stats.health.to_string());
                        ui.end_row();

                        ui.label("Stamina");
                        ui.label(&stats.stamina.to_string());
                        ui.end_row();

                        ui.label("Mana");
                        ui.label(&stats.mana.to_string());
                        ui.end_row();

                        ui.label("Attack");
                        ui.label(&stats.attack.to_string());
                        ui.end_row();

                        ui.label("Defense");
                        ui.label(&stats.defense.to_string());
                        ui.end_row();

                        ui.label("Magical attack");
                        ui.label(&stats.magical_attack.to_string());
                        ui.end_row();

                        ui.label("Magical defense");
                        ui.label(&stats.magical_defense.to_string());
                        ui.end_row();
                    });
                    ui.separator();

                    egui::Grid::new("character_points").show(ui, |ui| {
                        let entries = [
                            ("Strength", &mut player.points.strength),
                            ("Constitution", &mut player.points.constitution),
                            ("Intelligence", &mut player.points.intelligence),
                            ("Wisdom", &mut player.points.wisdom),
                            ("Stamina", &mut player.points.stamina),
                            ("Agility", &mut player.points.agility),
                            ("Dexterity", &mut player.points.dexterity),
                        ];

                        if player.unused_points == 0 {
                            for (label, value) in entries {
                                ui.label(label);
                                ui.label(&value.to_string());
                                ui.end_row();
                            }
                        } else {
                            let mut need_to_use_points = false;
                            for (label, value) in entries {
                                ui.label(label);
                                ui.horizontal(|ui| {
                                    ui.label(&value.to_string());
                                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                        if ui.button("+").clicked() {
                                            *value += 1;
                                            need_to_use_points = true;
                                        }
                                    });
                                });
                                ui.end_row();
                            }
                            if need_to_use_points {
                                player.use_stat_point();
                            }
                        }

                        ui.label("Points available");
                        ui.label(&player.unused_points.to_string());
                        ui.end_row();
                    });
                });
        }
        if env.inventory_window.is_displayed {
            egui::Window::new("Inventory")
                .collapsible(false)
                .open(&mut env.inventory_window.is_displayed)
                .show(&egui_ctx, |ui| {
                    egui::Grid::new("inventory").show(ui, |_ui| {});
                });
        }

        if env.character_window.is_displayed || env.inventory_window.is_displayed {
            let (_egui_output, paint_cmds) = egui_ctx.end_frame();
            let paint_jobs = egui_ctx.tessellate(paint_cmds);

            //Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
            //Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
            //drawing calls with it.
            //Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.
            unsafe {
                gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            }
            painter.paint_jobs(
                None,
                paint_jobs,
                &egui_ctx.texture(),
            );
        }

        system.clear();
        update_elapsed = if micro_elapsed < FRAME_DELAY {
            ::std::thread::sleep(Duration::from_micros((FRAME_DELAY - micro_elapsed) as u64));
            FRAME_DELAY
        } else {
            micro_elapsed
        };

        loop_timer = Instant::now();
        sort_update += 1;
        if sort_update > 10 {
            sort_update = 0;
        }
    }
}
