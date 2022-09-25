use bevy::core::CorePlugin;
use bevy::ecs::schedule::ShouldRun;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier2d::prelude::*;

use crate::{building, player};

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

use crate::{GameInfo, GameState, NotOutsideWorld, OutsideWorld, NOT_OUTSIDE_WORLD, OUTSIDE_WORLD};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_game)
                .with_system(animate_sprite)
                .with_system(player::player_movement_system.label("player_movement_system"))
                .with_system(player::animate_character_system.after("player_movement_system"))
                .with_system(update_camera)
                .with_system(update_text)
                .with_system(handle_input)
                .with_system(handle_windows),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(run_if_game)
                .with_system(handle_outside_events)
                .with_system(handle_inside_events),
        )
        .add_startup_system(setup_world)
        .add_startup_system(player::spawn_player)
        .add_startup_system(building::spawn_buildings);
    }
}

fn run_if_game(mode: Res<State<GameState>>) -> ShouldRun {
    if *mode.current() == GameState::Game {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

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

    for row in 0..4 {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform {
                    translation: Vec3 {
                        x: 200. + (30 * row) as f32,
                        y: 900. / 3.,
                        z: 0.,
                    },
                    ..default()
                },
                sprite: TextureAtlasSprite {
                    custom_size: Some(Vec2 { x: 26., y: 26. }),
                    ..default()
                },
                ..default()
            })
            .insert(AnimationHandler {
                timer: Timer::from_seconds(0.1, true),
                row,
            })
            .insert(OutsideWorld);
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
    mut camera: Query<(&mut Transform, &Camera), Without<player::Player>>,
    player: Query<(&Transform, &player::Player), Without<Camera>>,
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
    mut app_state: ResMut<GameInfo>,
    mut player: Query<&mut player::Player>,
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
    mut game_state: ResMut<State<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut app_state: ResMut<GameInfo>,
) {
    if keyboard_input.just_released(KeyCode::C) {
        app_state.show_character_window = !app_state.show_character_window;
    }
    if keyboard_input.just_released(KeyCode::Escape) {
        if app_state.show_character_window {
            app_state.show_character_window = false;
        } else {
            game_state.set(GameState::Menu).unwrap();
        }
    }
}

fn handle_outside_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut buildings: Query<(&mut building::House, &mut TextureAtlasSprite, &Children)>,
    mut visibilities: Query<(&mut Visibility), With<OutsideWorld>>,
    mut player: Query<(&Children, &mut Transform, &mut player::Player)>,
    mut collisions: Query<(&mut CollisionGroups)>,
    mut app_state: ResMut<GameInfo>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    if app_state.is_inside_building {
        return;
    }

    let player_id = match app_state.player_id {
        Some(x) => x,
        _ => return,
    };

    for collision_event in collision_events.iter() {
        println!("==> {:?}", collision_event);
        match collision_event {
            CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) => {
                let building_id = if *x == player_id {
                    y
                } else if *y == player_id {
                    x
                } else {
                    continue;
                };
                let elem1 = match collisions.get(*x) {
                    Ok(e) => e,
                    _ => continue,
                };
                let elem2 = match collisions.get(*y) {
                    Ok(e) => e,
                    _ => continue,
                };
                if elem1.memberships != elem2.memberships {
                    continue;
                }
                println!("Handling event!");
                for mut building in buildings.iter_mut() {
                    if building.2.contains(building_id) {
                        building.0.is_open = true;
                        building.0.contact_with_sensor += 1;
                        if building.0.contact_with_sensor > 1 {
                            // When we get out, the sensor will be triggered so we put back to 0.
                            building.0.contact_with_sensor = 0;
                            for mut visibility in visibilities.iter_mut() {
                                visibility.is_visible = false;
                            }
                            {
                                let (children, mut pos, mut player) = player.single_mut();
                                for child in children {
                                    if let Ok(mut collision) = collisions.get_mut(*child) {
                                        collision.memberships = NOT_OUTSIDE_WORLD;
                                        collision.filters = NOT_OUTSIDE_WORLD;
                                        break;
                                    }
                                }
                                player.old_x = pos.translation.x;
                                player.old_y = pos.translation.y - 10.;
                                pos.translation.x = -0.;
                                pos.translation.y = -30.;
                                app_state.is_inside_building = true;
                            }
                            building::spawn_inside_building(&mut commands, &asset_server);
                        } else {
                            building.1.index = building.0.is_open as _;
                        }
                        break;
                    }
                }
            }
            CollisionEvent::Stopped(x, y, CollisionEventFlags::SENSOR) => {
                let building_id = if *x == player_id {
                    y
                } else if *y == player_id {
                    x
                } else {
                    continue;
                };
                for mut building in buildings.iter_mut() {
                    if building.2.contains(building_id) {
                        building.0.contact_with_sensor =
                            building.0.contact_with_sensor.saturating_sub(1);
                        building.0.is_open = building.0.contact_with_sensor > 0;
                        building.1.index = building.0.is_open as _;
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_inside_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut visibilities: Query<&mut Visibility, With<OutsideWorld>>,
    mut inside_elems: Query<Entity, With<NotOutsideWorld>>,
    mut player: Query<(&Children, &mut Transform, &mut player::Player)>,
    mut collisions: Query<&mut CollisionGroups>,
    mut app_state: ResMut<GameInfo>,
    mut commands: Commands,
) {
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    if !app_state.is_inside_building {
        return;
    }

    let player_id = match app_state.player_id {
        Some(x) => x,
        _ => return,
    };

    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) => {
                if *x != player_id && *y != player_id {
                    continue;
                }
                let elem1 = match collisions.get(*x) {
                    Ok(e) => e,
                    _ => continue,
                };
                let elem2 = match collisions.get(*y) {
                    Ok(e) => e,
                    _ => continue,
                };
                if elem1.memberships != elem2.memberships {
                    continue;
                }
                println!(
                    "NEW ONE ==> {:?} {:?}",
                    collision_event,
                    player.single().1.translation
                );
                for mut visibility in visibilities.iter_mut() {
                    visibility.is_visible = true;
                }
                {
                    for entity in inside_elems.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    let (children, mut pos, mut player) = player.single_mut();
                    for child in children {
                        if let Ok(mut collision) = collisions.get_mut(*child) {
                            collision.memberships = OUTSIDE_WORLD;
                            collision.filters = OUTSIDE_WORLD;
                            break;
                        }
                    }
                    pos.translation.x = player.old_x;
                    pos.translation.y = player.old_y;
                    app_state.is_inside_building = false;
                }
            }
            _ => {}
        }
    }
}
