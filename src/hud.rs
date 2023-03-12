use bevy::prelude::*;

use crate::{character, player};

#[derive(Component)]
pub struct Hud;

#[derive(Component, Clone, Copy)]
pub enum StatKind {
    Health,
    Mana,
    Stamina,
}

fn spawn_stat_bar(commands: &mut Commands, stat: StatKind, background_color: BackgroundColor) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Px(102.0),
                    height: Val::Px(7.0),
                },
                border: UiRect::all(Val::Px(1.0)),
                position: UiRect {
                    left: Val::Px(4.0),
                    top: match stat {
                        StatKind::Health => Val::Px(4.0),
                        StatKind::Mana => Val::Px(12.0),
                        StatKind::Stamina => Val::Px(20.0),
                    },
                    ..default()
                },
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                        },
                        ..default()
                    },
                    background_color,
                    ..default()
                },
                stat,
                Hud,
            ));
        });
}

pub fn build_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_stat_bar(&mut commands, StatKind::Health, Color::RED.into());
    spawn_stat_bar(&mut commands, StatKind::Mana, Color::CYAN.into());
    spawn_stat_bar(&mut commands, StatKind::Stamina, Color::YELLOW.into());

    let font = asset_server.load(crate::FONT);
    let mut text_bundle = TextBundle::from_section(
        "",
        TextStyle {
            font,
            font_size: 40.0 / crate::SCALE,
            color: Color::WHITE,
        },
    )
    .with_text_alignment(TextAlignment::Right)
    .with_style(Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            top: Val::Px(2.0),
            right: Val::Px(2.0),
            ..default()
        },
        ..default()
    });
    text_bundle.visibility = Visibility::Hidden;
    commands.spawn((text_bundle, DebugText));
}

pub fn update_hud(
    character_info_updates: Query<
        &character::Character,
        (With<player::Player>, Changed<character::Character>),
    >,
    mut huds: Query<(&mut Style, &StatKind)>,
) {
    for character in character_info_updates.iter() {
        for (mut style, stat) in huds.iter_mut() {
            style.size.width = Val::Px(match stat {
                StatKind::Health => character.stats.health.pourcent(),
                StatKind::Mana => character.stats.mana.pourcent(),
                StatKind::Stamina => character.stats.stamina.pourcent(),
            });
        }
    }
}

#[derive(Component)]
pub struct DebugText;

pub fn update_text(
    mut text: Query<&mut Text, With<DebugText>>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera = camera.single().translation;
    text.single_mut().sections[0].value = format!("({:.2}, {:.2})", camera.x, camera.y);
}

pub fn run_if_debug(mode: Res<State<crate::DebugState>>) -> bool {
    mode.0 == crate::DebugState::Enabled
}
