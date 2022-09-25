#![allow(unused)]

mod building;
mod character;
mod game;
mod menu;
mod player;
mod stat;

use bevy::core::CorePlugin;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier2d::prelude::*;

pub const ONE_SECOND: u32 = 1_000_000;
pub const STAT_POINTS_PER_LEVEL: u32 = 3;

pub const OUTSIDE_WORLD: u32 = 1;
pub const NOT_OUTSIDE_WORLD: u32 = 2;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    Menu,
    Game,
}

#[derive(Component, Default)]
struct CameraPosition(Vec3);

#[derive(Default)]
pub struct GameInfo {
    pub show_character_window: bool,
    pub player_id: Option<Entity>,
    pub building_hash: u32,
}

pub fn despawn_kind<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn setup_components(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
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
        window.update_scale_factor_from_backend(1.8);
    }

    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow.extrusion = 0.;
    visuals.popup_shadow.extrusion = 0.;
    egui_context.ctx_mut().set_visuals(visuals);
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
        .insert_resource(GameInfo::default())
        .add_state(AppState::Game)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(EguiPlugin)
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_startup_system(setup_components)
        .run();
}
