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

fn spawn_stat_bar(commands: &mut Commands, stat: StatKind, color: UiColor) {
    commands
        .spawn_bundle(NodeBundle {
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
            color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                        },
                        ..default()
                    },
                    color,
                    ..default()
                })
                .insert(stat)
                .insert(Hud);
        });
}

pub fn build_stat_hud(
    mut commands: Commands,
    ass: ResMut<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_stat_bar(&mut commands, StatKind::Health, Color::RED.into());
    spawn_stat_bar(&mut commands, StatKind::Mana, Color::CYAN.into());
    spawn_stat_bar(&mut commands, StatKind::Stamina, Color::YELLOW.into());
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
