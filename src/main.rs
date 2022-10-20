#![allow(unused)]

mod building;
mod character;
mod environment;
mod game;
mod hud;
mod menu;
mod monster;
mod player;
mod stat;

use bevy::core::CorePlugin;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier2d::prelude::*;
// FIXME: to be removed once https://github.com/bevyengine/bevy/issues/1856 is fixed.
use bevy_pixel_camera::*;

pub const ONE_SECOND: u32 = 1_000_000;
pub const STAT_POINTS_PER_LEVEL: u32 = 3;

pub const OUTSIDE_WORLD: Group = Group::GROUP_1;
pub const NOT_OUTSIDE_WORLD: Group = Group::GROUP_2;
pub const RUN_STAMINA_CONSUMPTION_PER_SEC: f32 = 10.;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    Menu,
    Game,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum DebugState {
    Disabled,
    Enabled,
}

pub const SCALE: f32 = 1.8;

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

pub fn debug_disabled(
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut text: Query<&mut Visibility, With<hud::DebugText>>,
) {
    rapier_debug.enabled = false;
    if let Ok(mut text) = text.get_single_mut() {
        text.is_visible = false;
    }
}

pub fn debug_enabled(
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut text: Query<&mut Visibility, With<hud::DebugText>>,
) {
    rapier_debug.enabled = true;
    if let Ok(mut text) = text.get_single_mut() {
        text.is_visible = true;
    }
}

pub fn setup_components(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut egui_context: ResMut<EguiContext>,
    mut egui_settings: ResMut<bevy_egui::EguiSettings>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    // Disable gravity.
    rapier_config.gravity = Vec2::ZERO;

    // Add the 2D camera.
    // commands.spawn_bundle(Camera2dBundle::default());
    let resolution = (1600., 900.);
    commands.spawn_bundle(PixelCameraBundle::from_resolution(
        resolution.0 as _,
        resolution.1 as _,
    ));

    // Set the window size and its resolution.
    {
        let window = windows.get_primary_mut().unwrap();
        window.set_resolution(resolution.0, resolution.1);
        window.update_scale_factor_from_backend(SCALE as _);
    }

    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow.extrusion = 0.;
    visuals.popup_shadow.extrusion = 0.;
    egui_context.ctx_mut().set_visuals(visuals);
    egui_settings.scale_factor = 1. / SCALE as f64;

    // Setting up the debug display of the physics engine.
    // rapier_debug.enabled = false;
    rapier_debug.pipeline.mode = DebugRenderMode::from_bits_truncate(OUTSIDE_WORLD.bits());
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
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(PixelCameraPlugin)
        .add_plugin(PixelBorderPlugin {
            color: Color::rgb(0.1, 0.1, 0.1),
        })
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_startup_system(setup_components)
        .run();
}
