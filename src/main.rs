#![allow(unused)]

mod building;
mod character;
mod environment;
mod game;
mod hud;
mod inventory;
mod map;
mod menu;
mod monster;
mod player;
mod stat;
mod vendor;
mod weapon;

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImagePlugin;
use bevy::window::{PresentMode, WindowPlugin};
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier2d::prelude::*;
// FIXME: to be removed once https://github.com/bevyengine/bevy/issues/1856 is fixed.
// use bevy_pixel_camera::*;
use bevy_prototype_lyon::prelude::*;
use rand_pcg::Pcg64Mcg;

pub const ONE_SECOND: u32 = 1_000_000;
pub const STAT_POINTS_PER_LEVEL: u32 = 3;

pub const OUTSIDE_WORLD: Group = Group::GROUP_1;
pub const INTERACTION: Group = Group::GROUP_2;
pub const HITBOX: Group = Group::GROUP_3;
pub const NOTHING: Group = Group::GROUP_4;
pub const RUN_STAMINA_CONSUMPTION_PER_SEC: f32 = 10.;
pub const MAP_SIZE: f32 = 10_000.;

pub const SCALE: f32 = 1.8;

pub const WIDTH: f32 = 1600.;
pub const HEIGHT: f32 = 900.;

pub const FONT: &str = "fonts/kreon-regular.ttf";
pub const SEED: &str = "world";
pub type SeedType = Pcg64Mcg;

pub const GRASS_SIZE: f32 = 16.;
pub const BACKGROUND_Z_INDEX: f32 = 0.;
pub const CARPET_Z_INDEX: f32 = 0.1;
pub const CARPET_SYMBOL_Z_INDEX: f32 = 0.15;
pub const FURNITURE_Z_INDEX: f32 = 0.2;
// pub const WEAPON_Z_INDEX: f32 = 0.9;
pub const CHARACTER_Z_INDEX: f32 = 1.;
pub const FURNITURE_TOP_PART_Z_INDEX: f32 = 1.2;
pub const BUILDING_TOP_PART_Z_INDEX: f32 = 1.8;

pub const CYAN: Color = Color::LinearRgba(LinearRgba::rgb(0., 1., 1.));
pub const YELLOW: Color = Color::LinearRgba(LinearRgba::rgb(1., 1., 0.));
pub const CRIMSON: Color = Color::LinearRgba(LinearRgba::rgb(0.86, 0.08, 0.24));

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AppState {
    Menu,
    #[default]
    Game,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum DebugState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Default, Resource)]
pub struct GameInfo {
    pub show_character_window: bool,
    pub show_inventory_window: bool,
    pub player_id: Option<Entity>,
    pub building_hash: u32,
    pub building: Option<building::Building>,
    pub pos: Vec2,
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
    if let Ok(mut text_visibility) = text.get_single_mut() {
        *text_visibility = Visibility::Hidden;
    }
}

pub fn debug_enabled(
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut text: Query<&mut Visibility, With<hud::DebugText>>,
) {
    rapier_debug.enabled = true;
    if let Ok(mut text_visibility) = text.get_single_mut() {
        *text_visibility = Visibility::Inherited;
    }
}

pub fn setup_components(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut egui_context: EguiContexts,
    mut egui_settings: ResMut<bevy_egui::EguiSettings>,
    mut rapier_debug: ResMut<DebugRenderContext>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Disable gravity.
    rapier_config.gravity = Vec2::ZERO;

    // Add the 2D camera.
    commands.spawn(Camera2dBundle::default());
    let resolution = (WIDTH, HEIGHT);
    // commands.spawn(PixelCameraBundle::from_resolution(
    //     resolution.0 as _,
    //     resolution.1 as _,
    // ));

    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow.spread = 0.;
    visuals.popup_shadow.spread = 0.;
    egui_context.ctx_mut().set_visuals(visuals);
    egui_settings.scale_factor = 1. / SCALE;

    // Setting up the debug display of the physics engine.
    // rapier_debug.enabled = false;
    rapier_debug.pipeline.mode = DebugRenderMode::from_bits_truncate(OUTSIDE_WORLD.bits())
        .union(DebugRenderMode::from_bits_truncate(HITBOX.bits()));
}

fn main() {
    let mut app = App::new();

    // let options = app
    //     .world
    //     .get_resource::<bevy::render::settings::WgpuSettings>()
    //     .cloned()
    //     .unwrap_or_default();

    // let instance = wgpu::Instance::new(bevy::render::settings::Backends::VULKAN);
    // let request_adapter_options = wgpu::RequestAdapterOptions {
    //     power_preference: options.power_preference,
    //     ..Default::default()
    // };
    // futures_lite::future::block_on(bevy::render::renderer::initialize_renderer(
    //     &instance,
    //     &options,
    //     &request_adapter_options,
    // ));

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Toy game".to_string(),
                    present_mode: PresentMode::AutoVsync,
                    resizable: false,
                    resolution: WindowResolution::new(WIDTH, HEIGHT)
                        .with_scale_factor_override(SCALE),
                    ..default()
                }),
                ..default()
            })
            // prevents blurry sprites
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(GameInfo::default())
    .init_state::<AppState>()
    .add_plugins((
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
        EguiPlugin,
        RapierDebugRenderPlugin::default(),
        // PixelCameraPlugin,
        // .PixelBorderPlugin {
        //     color: LinearRgba::rgb(0.1, 0.1, 0.1),
        // },
        ShapePlugin,
        menu::MenuPlugin,
        game::GamePlugin,
    ))
    .add_systems(Startup, (setup_components))
    .run();
}
