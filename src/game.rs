use bevy::core::CorePlugin;
use bevy::ecs::schedule::ShouldRun;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier2d::prelude::*;

use crate::{
    building, character, environment, hud, monster, player, AppState, GameInfo, NOT_OUTSIDE_WORLD,
    OUTSIDE_WORLD,
};

pub const ONE_SECOND: u32 = 1_000_000;
pub const STAT_POINTS_PER_LEVEL: u32 = 3;

pub struct GamePlugin;

#[derive(Component)]
pub struct InsideHouse;
#[derive(Component)]
pub struct OutsideWorld;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Outside,
    /// Contains the hash to be used to generate the inside.
    InsideHouse,
    /// Contains the hash to be used to generate the inside.
    InsideDungeon,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_game)
                .with_system(player::player_movement_system.label("player_movement_system"))
                .with_system(character::animate_character_system.after("player_movement_system"))
                .with_system(hud::update_hud.after("player_movement_system"))
                .with_system(update_camera.after("player_movement_system"))
                .with_system(handle_input)
                .with_system(handle_windows),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(hud::run_if_debug)
                .with_system(hud::update_text.after("player_movement_system")),
        )
        .add_system_set(
            SystemSet::on_enter(crate::DebugState::Disabled).with_system(crate::debug_disabled),
        )
        .add_system_set(
            SystemSet::on_exit(crate::DebugState::Disabled).with_system(crate::debug_enabled),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::InsideHouse)
                .with_run_criteria(run_if_game)
                .with_system(handle_door_events::<InsideHouse>)
                .with_system(handle_enter_area_events::<InsideHouse>),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::on_update(GameState::Outside)
                .with_run_criteria(run_if_game)
                .with_system(handle_door_events::<OutsideWorld>)
                .with_system(handle_enter_area_events::<OutsideWorld>),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Game)
                .with_system(player::spawn_player)
                .with_system(monster::spawn_monsters)
                .with_system(building::spawn_buildings)
                .with_system(environment::spawn_nature)
                .with_system(hud::build_hud),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::InsideHouse)
                .with_system(building::spawn_inside_building)
                .with_system(hide_outside),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::InsideHouse)
                .with_system(crate::despawn_kind::<InsideHouse>)
                .with_system(show_outside),
        )
        .add_state(GameState::Outside)
        .add_state(crate::DebugState::Disabled);
    }
}

fn run_if_game(mode: Res<State<AppState>>) -> ShouldRun {
    if *mode.current() == AppState::Game {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (Without<player::Player>, With<Camera>)>,
    player: Query<&Transform, (Without<Camera>, With<player::Player>, Changed<Transform>)>,
) {
    let player = match player.get_single() {
        Ok(p) => p,
        _ => return,
    };
    let mut camera = camera.single_mut();

    camera.translation.x = player.translation.x;
    camera.translation.y = player.translation.y;
}

fn handle_windows(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<GameInfo>,
    mut player: Query<&mut character::Character, With<player::Player>>,
) {
    if !app_state.show_character_window {
        return;
    }
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
                let character = player.single();

                ui.label("Level");
                ui.label(&character.level.to_string());
                ui.end_row();

                ui.label("Experience");
                ui.label(&format!(
                    "{} / {}",
                    character.xp, character.xp_to_next_level
                ));
                ui.end_row();

                let stats = &character.stats;

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
                let character = player.single();

                let entries = [
                    ("Strength", character.points.strength),
                    ("Constitution", character.points.constitution),
                    ("Intelligence", character.points.intelligence),
                    ("Wisdom", character.points.wisdom),
                    ("Stamina", character.points.stamina),
                    ("Agility", character.points.agility),
                    ("Dexterity", character.points.dexterity),
                ];
                let mut updates = Vec::with_capacity(entries.len());

                let unused_points = character.unused_points;
                // drop(character);
                if unused_points == 0 {
                    for (label, value) in entries {
                        ui.label(label);
                        ui.label(&value.to_string());
                        ui.end_row();
                    }
                } else {
                    let mut need_to_use_points = false;
                    for (pos, (label, value)) in entries.into_iter().enumerate() {
                        ui.label(label);
                        ui.horizontal(|ui| {
                            ui.label(&value.to_string());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if ui.button("+").clicked() {
                                    updates.push(pos);
                                    need_to_use_points = true;
                                }
                            });
                        });
                        ui.end_row();
                    }
                    if need_to_use_points {
                        // We do it in two steps to avoid triggering a `Changed` event to the HUD.
                        let mut character = &mut *player.single_mut();
                        character.use_stat_point();
                        let parts = [
                            &mut character.points.strength,
                            &mut character.points.constitution,
                            &mut character.points.intelligence,
                            &mut character.points.wisdom,
                            &mut character.points.stamina,
                            &mut character.points.agility,
                            &mut character.points.dexterity,
                        ];
                        for update in updates {
                            *parts[update] += 1;
                        }
                    }
                }

                ui.label("Points available");
                ui.label(&unused_points.to_string());
                ui.end_row();
            });
        });
}

pub fn handle_input(
    mut game_state: ResMut<State<AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut app_state: ResMut<GameInfo>,
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut debug_state: ResMut<State<crate::DebugState>>,
) {
    if keyboard_input.just_released(KeyCode::C) {
        app_state.show_character_window = !app_state.show_character_window;
    }
    if keyboard_input.just_released(KeyCode::Escape) {
        if app_state.show_character_window {
            app_state.show_character_window = false;
        } else {
            game_state.push(AppState::Menu).unwrap();
        }
    }
    if keyboard_input.just_released(KeyCode::F5) {
        if *debug_state.current() == crate::DebugState::Enabled {
            debug_state.set(crate::DebugState::Disabled);
        } else {
            debug_state.set(crate::DebugState::Enabled);
        }
    }
}

#[inline]
fn update_player_collisions(
    player: &mut Query<(&Children, &mut Transform, &mut player::Player)>,
    collisions: &mut Query<&mut CollisionGroups>,
    filter: Group,
) {
    let (children, mut pos, mut player) = player.single_mut();
    for child in children {
        if let Ok(mut collision) = collisions.get_mut(*child) {
            collision.memberships = filter;
            collision.filters = filter;
            break;
        }
    }
}

fn hide_outside(
    mut visibilities: Query<(&mut Visibility), With<OutsideWorld>>,
    mut player: Query<(&Children, &mut Transform, &mut player::Player)>,
    mut collisions: Query<&mut CollisionGroups>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    for mut visibility in visibilities.iter_mut() {
        visibility.is_visible = false;
    }

    // FIXME: https://github.com/dimforge/rapier/issues/398
    // rapier_debug.pipeline.mode = DebugRenderMode::from_bits_truncate(NOT_OUTSIDE_WORLD.bits());
    update_player_collisions(&mut player, &mut collisions, NOT_OUTSIDE_WORLD);

    let (children, mut pos, mut player) = player.single_mut();
    player.old_x = pos.translation.x;
    player.old_y = pos.translation.y - 10.;
    pos.translation.x = -0.;
    pos.translation.y = -40.;
}

fn show_outside(
    mut visibilities: Query<(&mut Visibility), With<OutsideWorld>>,
    mut player: Query<(&Children, &mut Transform, &mut player::Player)>,
    mut collisions: Query<&mut CollisionGroups>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    for mut visibility in visibilities.iter_mut() {
        visibility.is_visible = true;
    }

    // FIXME: https://github.com/dimforge/rapier/issues/398
    // rapier_debug.pipeline.mode = DebugRenderMode::from_bits_truncate(OUTSIDE_WORLD.bits());
    update_player_collisions(&mut player, &mut collisions, OUTSIDE_WORLD);

    let (children, mut pos, mut player) = player.single_mut();
    pos.translation.x = player.old_x;
    pos.translation.y = player.old_y;
}

macro_rules! get_building_and_player {
    ($x:ident, $y:ident, $player_id:ident, $buildings:ident, $door_captors:ident, $value:expr) => {
        let building_id = if *$x == $player_id {
            $y
        } else if *$y == $player_id {
            $x
        } else {
            continue;
        };
        if !$door_captors.contains(*building_id) {
            continue;
        }
        for mut building in $buildings.iter_mut() {
            if building.1.contains(building_id) {
                building.0.index = $value;
                break;
            }
        }
    };
}

fn handle_door_events<T: Component>(
    mut collision_events: EventReader<CollisionEvent>,
    mut buildings: Query<(&mut TextureAtlasSprite, &Children, &T), With<building::House>>,
    door_captors: Query<&building::Door>,
    app_state: ResMut<GameInfo>,
) {
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    let player_id = match app_state.player_id {
        Some(x) => x,
        _ => return,
    };

    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) => {
                get_building_and_player!(x, y, player_id, buildings, door_captors, 1);
            }
            CollisionEvent::Stopped(x, y, CollisionEventFlags::SENSOR) => {
                get_building_and_player!(x, y, player_id, buildings, door_captors, 0);
            }
            _ => {}
        }
    }
}

fn handle_enter_area_events<T: Component>(
    mut collision_events: EventReader<CollisionEvent>,
    buildings: Query<&Children, With<T>>,
    enter_area_captors: Query<&building::EnterArea>,
    mut app_state: ResMut<GameInfo>,
    mut game_state: ResMut<State<GameState>>,
) {
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    let player_id = match app_state.player_id {
        Some(x) => x,
        _ => return,
    };

    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) = collision_event {
            let building_id = if *x == player_id {
                y
            } else if *y == player_id {
                x
            } else {
                continue;
            };
            if !enter_area_captors.contains(*building_id) {
                continue;
            }
            for mut building in buildings.iter() {
                if building.contains(building_id) {
                    // FIXME: compute real hash
                    app_state.building_hash = 0;
                    if *game_state.current() == GameState::Outside {
                        game_state.set(GameState::InsideHouse);
                    } else {
                        game_state.set(GameState::Outside);
                    }
                    return;
                }
            }
        }
    }
}
