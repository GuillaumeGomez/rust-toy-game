#![allow(unused)]

mod character;
mod player;
mod stat;

use bevy::core::CorePlugin;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_rapier2d::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

pub const ONE_SECOND: u32 = 1_000_000;
pub const STAT_POINTS_PER_LEVEL: u32 = 3;

#[derive(Component)]
struct AnimationHandler {
    timer: Timer,
    row: usize,
}
#[derive(Component)]
struct EnableText;
#[derive(Component, Default)]
struct CameraPosition(Vec3);

fn setup_world(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut egui_context: ResMut<EguiContext>,
) {
    // Disable gravity.
    rapier_config.gravity = Vec2::ZERO;

    // Add the 2D camera/
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(CameraPosition::default());

    // Set the window size and its resolution.
    {
        let window = windows.get_primary_mut().unwrap();
        window.set_resolution(1600., 900.);
        window.update_scale_factor_from_backend(1.0);
    }

    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow.extrusion = 0.;
    visuals.popup_shadow.extrusion = 0.;
    egui_context.ctx_mut().set_visuals(visuals);

    let skeleton_texture = asset_server.load("textures/skeleton.png");
    let texture_atlas = TextureAtlas::from_grid(skeleton_texture, Vec2::new(48., 48.), 3, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let wrap = (1600. - 200.) / 4.;
    for row in 0..4 {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform {
                    translation: Vec3 {
                        x: 1400. / -2. + wrap * (row as f32),
                        y: 900. / 3.,
                        z: 0.,
                    },
                    ..default()
                },
                ..default()
            })
            .insert(AnimationHandler {
                timer: Timer::from_seconds(0.1, true),
                row,
            });
    }
    let font = asset_server.load("fonts/kreon-regular.ttf");
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(
                "hello",
                TextStyle {
                    font,
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            ..default()
        })
        .insert(EnableText);
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationHandler,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut handler, mut sprite, texture_atlas_handle) in &mut query {
        handler.timer.tick(time.delta());
        if handler.timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % 3 + handler.row * 3;
        }
    }
}

fn update_camera(
    mut camera: Query<(&mut Transform, &Camera), Without<player::PlayerComponent>>,
    player: Query<(&Transform, &player::PlayerComponent), Without<Camera>>,
) {
    let player = player.single();
    let mut camera = camera.single_mut().0;

    camera.translation.x = player.0.translation.x;
    camera.translation.y = player.0.translation.y;
}

fn update_text(
    mut text: Query<&mut Text, With<EnableText>>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera = camera.single().translation;
    text.single_mut().sections[0].value = format!("({:.2}, {:.2})", camera.x, camera.y);
}

fn handle_windows(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<GameState>,
    mut player: Query<&mut player::PlayerComponent>,
) {
    if !app_state.show_character_window {
        return;
    }
    let player = &mut player.single_mut().character;
    egui::Window::new("Character information")
        .collapsible(false)
        .resizable(false)
        .open(&mut app_state.show_character_window)
        .show(egui_context.ctx_mut(), |ui| {
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
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
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

pub fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_state: ResMut<GameState>,
) {
    if keyboard_input.just_released(KeyCode::C) {
        app_state.show_character_window = !app_state.show_character_window;
    }
    if keyboard_input.just_released(KeyCode::Escape) {
        app_state.show_character_window = false;
    }
}

#[derive(Default)]
pub struct GameState {
    pub show_character_window: bool,
}

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest()) // prevents blurry sprites
        .insert_resource(WindowDescriptor {
            title: "Toy game".to_string(),
            present_mode: PresentMode::AutoVsync,
            resizable: false,
            ..default()
        })
        .insert_resource(GameState::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(EguiPlugin)
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_world)
        .add_startup_system(player::spawn_player)
        .add_system(animate_sprite)
        .add_system(player::player_movement_system.label("player_movement_system"))
        .add_system(player::animate_character_system.after("player_movement_system"))
        .add_system(update_camera)
        .add_system(update_text)
        .add_system(handle_input)
        .add_system(handle_windows)
        .run();
}
