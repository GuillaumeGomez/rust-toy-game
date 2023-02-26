use bevy::core::CorePlugin;
use bevy::ecs::schedule::ShouldRun;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier2d::prelude::*;
use once_cell::sync::Lazy;

use crate::{
    building, character, environment, hud, map, monster, player, weapon, AppState, GameInfo,
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
                .with_system(player::player_attack_system.before("player_movement_system"))
                .with_system(player::player_movement_system.label("player_movement_system"))
                .with_system(weapon::handle_attacks.after("player_movement_system"))
                .with_system(character::animate_character_system.after("player_movement_system"))
                .with_system(character::refresh_characters_stats.after("player_movement_system"))
                .with_system(hud::update_hud.after("player_movement_system"))
                .with_system(update_camera.after("player_movement_system"))
                .with_system(weapon::update_notifications)
                .with_system(monster::update_character_info)
                .with_system(environment::grass_events)
                .with_system(weapon::update_entity_destroyer)
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
                .with_system(map::spawn_map.label("spawn_map"))
                .with_system(player::spawn_player.after("spawn_map"))
                .with_system(monster::spawn_monsters.after("spawn_map"))
                // TODO: move this into `spawn_map`
                .with_system(building::spawn_buildings)
                // TODO: move this into `spawn_map`
                .with_system(building::spawn_statues)
                // TODO: move this into `spawn_map`
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

fn show_character_window(
    egui_context: &mut ResMut<EguiContext>,
    app_state: &mut ResMut<GameInfo>,
    player: &mut Query<&mut character::Character, With<player::Player>>,
) {
    egui::Window::new("Character information")
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::Pos2::new(2., crate::HEIGHT / 4.))
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

const INVENTORY_LINE_SIZE: usize = 5;
const INVENTORY_NB_LINE: usize = 7;

fn show_inventory_window(
    egui_context: &mut ResMut<EguiContext>,
    app_state: &mut ResMut<GameInfo>,
    asset_server: Res<AssetServer>,
) {
    static IDS: Lazy<Vec<egui::Id>> = Lazy::new(|| {
        let mut v = Vec::with_capacity(INVENTORY_NB_LINE * INVENTORY_LINE_SIZE);

        for y in 0..INVENTORY_NB_LINE {
            for x in 0..INVENTORY_LINE_SIZE {
                v.push(egui::Id::new(format!("inv {y}:{x}")));
            }
        }
        v
    });
    let weapon_handle = asset_server.load("textures/weapon.png");
    let image_id = egui_context.add_image(weapon_handle);

    let image_vec = egui::Vec2::new(7., 20.);
    let image = egui::Image::new(image_id, image_vec);

    const CASE_SIZE: f32 = 40.;

    let mut ids = IDS.iter();
    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::Pos2::new(crate::WIDTH - 10., crate::HEIGHT / 4.))
        .open(&mut app_state.show_inventory_window)
        .show(egui_context.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("inventory")
                    .spacing(egui::Vec2::new(4., 4.))
                    .show(ui, |ui| {
                        for y in 0..INVENTORY_NB_LINE {
                            for x in 0..INVENTORY_LINE_SIZE {
                                let (rect, _) = ui.allocate_at_least(
                                    egui::Vec2::new(CASE_SIZE + 2., CASE_SIZE + 2.),
                                    egui::Sense::hover(),
                                );
                                let res = ui.interact(
                                    rect,
                                    *ids.next().unwrap(),
                                    egui::Sense::click_and_drag(),
                                );
                                let stroke_color = if res.hovered() {
                                    egui::Color32::LIGHT_RED
                                } else {
                                    egui::Color32::WHITE
                                };
                                ui.painter().rect(
                                    rect,
                                    0.,
                                    egui::Color32::from_gray(52),
                                    egui::Stroke::new(1., stroke_color),
                                );
                                if y == 0 && x == 0 {
                                    let mut draw = rect;
                                    let center = rect.center();
                                    draw.min.x = center.x - image_vec.x / 2.;
                                    draw.min.y = center.y - image_vec.y / 2.;
                                    draw.max.x = center.x + image_vec.x / 2.;
                                    draw.max.y = center.y + image_vec.y / 2.;
                                    image.paint_at(ui, draw);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
        });
}

fn handle_windows(
    mut egui_context: ResMut<EguiContext>,
    mut app_state: ResMut<GameInfo>,
    mut player: Query<&mut character::Character, With<player::Player>>,
    asset_server: Res<AssetServer>,
) {
    if app_state.show_character_window {
        show_character_window(&mut egui_context, &mut app_state, &mut player);
    }
    if app_state.show_inventory_window {
        show_inventory_window(&mut egui_context, &mut app_state, asset_server);
    }
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
    if keyboard_input.just_released(KeyCode::I) {
        app_state.show_inventory_window = !app_state.show_inventory_window;
    }
    if keyboard_input.just_released(KeyCode::Escape) {
        if app_state.show_character_window {
            app_state.show_character_window = false;
        } else if app_state.show_inventory_window {
            app_state.show_inventory_window = false;
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
            if collision.memberships != crate::HITBOX {
                collision.memberships = filter;
                collision.filters = filter;
                break;
            }
        }
    }
}

fn hide_outside(
    mut player: Query<(&Children, &mut Transform, &mut player::Player)>,
    app_state: Res<GameInfo>,
) {
    let (children, mut pos, mut player) = player.single_mut();
    player.old_x = pos.translation.x;
    player.old_y = pos.translation.y - 10.;
    pos.translation.x = app_state.pos.x;
    pos.translation.y = app_state.pos.y - 40.;
}

fn show_outside(mut player: Query<(&Children, &mut Transform, &mut player::Player)>) {
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
    mut buildings: Query<(&mut TextureAtlasSprite, &Children, &building::Building, &T)>,
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
    buildings: Query<(&Children, &crate::building::Building), With<T>>,
    enter_area_captors: Query<&building::EnterArea>,
    player: Query<&Transform, With<player::Player>>,
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
            for (mut children, building) in buildings.iter() {
                if children.contains(building_id) {
                    // FIXME: compute real hash
                    app_state.building_hash = 0;
                    if *game_state.current() == GameState::Outside {
                        let player_pos = player.single();
                        app_state.pos = Vec2 {
                            x: player_pos.translation.x + crate::MAP_SIZE * 3.,
                            y: player_pos.translation.y + crate::MAP_SIZE * 3.,
                        };
                        app_state.building = Some(*building);
                        game_state.set(GameState::InsideHouse);
                    } else {
                        app_state.building = None;
                        game_state.set(GameState::Outside);
                    }
                    return;
                }
            }
        }
    }
}
