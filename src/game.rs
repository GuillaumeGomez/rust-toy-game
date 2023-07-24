use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier2d::prelude::*;
use once_cell::sync::Lazy;

use crate::menu::MenuState;
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Outside,
    /// Contains the hash to be used to generate the inside.
    InsideHouse,
    /// Contains the hash to be used to generate the inside.
    InsideDungeon,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_state::<crate::DebugState>()
            .add_systems(
                Update,
                (player::player_attack_system,)
                    .run_if(in_state(MenuState::Disabled))
                    .before(player::player_movement_system),
            )
            .add_systems(
                Update,
                (
                    weapon::handle_attacks,
                    character::animate_character_system,
                    character::refresh_characters_stats,
                    hud::update_hud,
                    update_camera,
                )
                    .run_if(in_state(MenuState::Disabled))
                    .after(player::player_movement_system),
            )
            .add_systems(
                Update,
                (
                    player::player_movement_system,
                    character::interaction_events,
                    weapon::update_notifications,
                    monster::update_character_info,
                    environment::grass_events,
                    weapon::update_entity_destroyer,
                    handle_input,
                    handle_windows,
                )
                    .run_if(in_state(MenuState::Disabled)),
            )
            .add_systems(
                Update,
                (hud::update_text)
                    .run_if(hud::run_if_debug)
                    .after(player::player_movement_system),
            )
            .add_systems(
                OnEnter(crate::DebugState::Disabled),
                (crate::debug_disabled),
            )
            .add_systems(OnExit(crate::DebugState::Disabled), (crate::debug_enabled))
            .add_systems(
                Update,
                (
                    handle_door_events::<InsideHouse>,
                    handle_enter_area_events::<InsideHouse>,
                )
                    .run_if(in_state(MenuState::Disabled))
                    .run_if(in_state(GameState::InsideHouse)),
            )
            .add_systems(
                Update,
                (
                    handle_door_events::<OutsideWorld>,
                    handle_enter_area_events::<OutsideWorld>,
                )
                    .run_if(in_state(MenuState::Disabled))
                    .run_if(in_state(GameState::Outside)),
            )
            .add_systems(
                OnEnter(AppState::Game),
                (
                    map::spawn_map,
                    player::spawn_player.after(map::spawn_map),
                    monster::spawn_monsters.after(map::spawn_map),
                    // TODO: move this into `spawn_map`
                    building::spawn_buildings,
                    // TODO: move this into `spawn_map`
                    building::spawn_statues,
                    // TODO: move this into `spawn_map`
                    environment::spawn_nature,
                    hud::build_hud,
                ),
            )
            .add_systems(
                OnEnter(GameState::InsideHouse),
                (building::spawn_inside_building, hide_outside),
            )
            .add_systems(
                OnExit(GameState::InsideHouse),
                (crate::despawn_kind::<InsideHouse>, show_outside),
            );
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
    egui_context: &mut EguiContexts,
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
    egui_context: &mut EguiContexts,
    app_state: &mut ResMut<GameInfo>,
    asset_server: Res<AssetServer>,
    player_inventory: Query<&crate::inventory::Inventory, With<crate::player::Player>>,
) {
    let inventory = match player_inventory.get_single() {
        Ok(i) => i,
        _ => return,
    };
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
    let weapon_image_id = egui_context.add_image(weapon_handle);

    let coin_handle = asset_server.load("textures/gold-coin.png");
    let coin_image_id = egui_context.add_image(coin_handle);

    const CASE_SIZE: f32 = 40.;
    const WIDTH: f32 = INVENTORY_LINE_SIZE as f32 * (CASE_SIZE + 2. + SPACING) - SPACING;
    const SPACING: f32 = 4.;

    static EQUIPMENT_SLOTS: Lazy<Vec<egui::Rect>> = Lazy::new(|| {
        let size = egui::Vec2::new(CASE_SIZE + 2., CASE_SIZE + 2.);
        let middle = WIDTH / 2. - CASE_SIZE / 2.;
        vec![
            // head
            egui::Rect::from_min_size(egui::Pos2::new(middle, 4.), size),
            // weapon
            egui::Rect::from_min_size(egui::Pos2::new(4., CASE_SIZE + 14.), size),
            // armor
            egui::Rect::from_min_size(egui::Pos2::new(middle, CASE_SIZE + 14.), size),
            // shoes
            egui::Rect::from_min_size(egui::Pos2::new(middle, (CASE_SIZE + 10.) * 2. + 4.), size),
        ]
    });

    const EQUIPMENT_HEIGHT: f32 = (CASE_SIZE + 10.) * 3. + 8.;
    const INVENTORY_HEIGHT: f32 = INVENTORY_NB_LINE as f32 * (CASE_SIZE + 2. + SPACING) - SPACING;
    const PIECE_SIZE: f32 = 15.;

    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(false)
        .default_pos(egui::Pos2::new(
            crate::WIDTH - WIDTH - 30.,
            crate::HEIGHT / 4.,
        ))
        .fixed_size(egui::Vec2::new(
            WIDTH,
            EQUIPMENT_HEIGHT + INVENTORY_HEIGHT + PIECE_SIZE + 10.,
        ))
        .open(&mut app_state.show_inventory_window)
        .show(egui_context.ctx_mut(), |ui| {
            // Equipment.
            let (rect, _) = ui.allocate_exact_size(
                egui::Vec2::new(WIDTH - 20., EQUIPMENT_HEIGHT),
                egui::Sense::hover(),
            );
            for pos in EQUIPMENT_SLOTS.iter() {
                ui.put(
                    pos.translate(egui::Vec2::new(rect.min.x, rect.min.y)),
                    |ui: &mut egui::Ui| {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::Vec2::new(CASE_SIZE + 2., CASE_SIZE + 2.),
                            egui::Sense::click_and_drag(),
                        );
                        let stroke_color = if response.hovered() {
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
                        response
                    },
                );
            }

            // Separator
            ui.separator();

            // Inventory.
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("inventory")
                    .spacing(egui::Vec2::new(SPACING, SPACING))
                    .show(ui, |ui| {
                        let image_vec = egui::Vec2::new(7., 20.);
                        let image = egui::Image::new(weapon_image_id, image_vec);

                        for y in 0..INVENTORY_NB_LINE {
                            for x in 0..INVENTORY_LINE_SIZE {
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::Vec2::new(CASE_SIZE + 2., CASE_SIZE + 2.),
                                    egui::Sense::click_and_drag(),
                                );
                                let stroke_color = if response.hovered() {
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

            // Money
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                let image_vec = egui::Vec2::new(PIECE_SIZE, PIECE_SIZE);
                let image = egui::Image::new(coin_image_id, image_vec);

                let (rect, _) = ui.allocate_at_least(
                    egui::Vec2::new(PIECE_SIZE, PIECE_SIZE),
                    egui::Sense::hover(),
                );
                image.paint_at(ui, rect);

                ui.label(&inventory.gold.to_string());
            });
        });
}

fn handle_windows(
    mut egui_context: EguiContexts,
    mut app_state: ResMut<GameInfo>,
    mut player: Query<&mut character::Character, With<player::Player>>,
    asset_server: Res<AssetServer>,
    player_inventory: Query<&crate::inventory::Inventory, With<crate::player::Player>>,
) {
    if app_state.show_character_window {
        show_character_window(&mut egui_context, &mut app_state, &mut player);
    }
    if app_state.show_inventory_window {
        show_inventory_window(
            &mut egui_context,
            &mut app_state,
            asset_server,
            player_inventory,
        );
    }
}

pub fn handle_input(
    mut menu_state: ResMut<NextState<MenuState>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut app_state: ResMut<GameInfo>,
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut debug_state: ResMut<State<crate::DebugState>>,
    mut next_debug_state: ResMut<NextState<crate::DebugState>>,
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
            menu_state.set(MenuState::Main);
        }
    }
    if keyboard_input.just_released(KeyCode::F5) {
        if *debug_state == crate::DebugState::Enabled {
            next_debug_state.set(crate::DebugState::Disabled);
        } else {
            next_debug_state.set(crate::DebugState::Enabled);
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
    game_state: ResMut<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
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
                    if *game_state == GameState::Outside {
                        let player_pos = player.single();
                        app_state.pos = Vec2 {
                            x: player_pos.translation.x + crate::MAP_SIZE * 3.,
                            y: player_pos.translation.y + crate::MAP_SIZE * 3.,
                        };
                        app_state.building = Some(*building);
                        next_game_state.set(GameState::InsideHouse);
                    } else {
                        app_state.building = None;
                        next_game_state.set(GameState::Outside);
                    }
                    return;
                }
            }
        }
    }
}
