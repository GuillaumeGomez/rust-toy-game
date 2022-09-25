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

#[derive(Component)]
pub struct OutsideWorld;
#[derive(Component)]
pub struct NotOutsideWorld;

pub const OUTSIDE_WORLD: u32 = 1;
pub const NOT_OUTSIDE_WORLD: u32 = 2;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Menu,
    Game,
}

#[derive(Default)]
pub struct GameInfo {
    pub show_character_window: bool,
    pub player_id: Option<Entity>,
    pub is_inside_building: bool,
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
        .add_state(GameState::Game)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(EguiPlugin)
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .run();
}
