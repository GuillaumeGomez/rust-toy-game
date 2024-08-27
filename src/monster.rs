use bevy::prelude::*;
use bevy_prototype_lyon::draw;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterHealthBar,
    CharacterHealthBarInner, CharacterInfo, CharacterKind, CharacterPoints, GrassEffectBundle,
};
use crate::game::OutsideWorld;

#[derive(Component)]
pub struct Skeleton;

const WIDTH: f32 = 26.;
const HEIGHT: f32 = 26.;

pub fn spawn_monsters(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    const NB_ANIMATIONS: usize = 3;
    const ANIMATION_TIME: f32 = 0.15;

    let skeleton_texture = asset_server.load("textures/skeleton.png");
    let texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(48, 48), NB_ANIMATIONS as _, 4, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let level = 2;

    commands
        .spawn((
            Skeleton,
            crate::inventory::Inventory {
                items: Vec::new(),
                gold: 1, // To be computed based on the monster level, etc.
                equipped_weapon: None,
            },
            Character::new(
                level,
                0,
                // FIXME, should have a method for a specific level.
                CharacterPoints::level_1(),
                WIDTH,
                HEIGHT,
                CharacterKind::Monster,
            ),
            CharacterAnimationInfo::new(
                ANIMATION_TIME,
                NB_ANIMATIONS,
                CharacterAnimationType::ForwardIdle,
            ),
            TextureAtlas {
                layout: texture_atlas_handle,
                ..default()
            },
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2 {
                        x: WIDTH,
                        y: HEIGHT,
                    }),
                    ..default()
                },
                texture: skeleton_texture,
                transform: Transform::from_xyz(200.0, 210.0, crate::CHARACTER_Z_INDEX),
                ..default()
            },
            RigidBody::Dynamic,
            Velocity::zero(),
            LockedAxes::ROTATION_LOCKED,
            Damping {
                linear_damping: 8.,
                angular_damping: 8.,
            },
            OutsideWorld,
        ))
        .with_children(|children| {
            // move box
            children.spawn((
                Collider::cuboid(8.0, 7.0),
                TransformBundle::from(Transform::from_xyz(0.0, -5.0, 0.0)),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
            ));
            children.spawn((
                Collider::cuboid(WIDTH / 2. - 6., HEIGHT / 2. - 1.),
                TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
                Sensor,
                CollisionGroups::new(crate::HITBOX, crate::HITBOX),
            ));

            children.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        &format!("Skeleton lvl. {}", level),
                        TextStyle {
                            font: asset_server.load(crate::FONT),
                            font_size: 10.0,
                            color: Color::LinearRgba(LinearRgba::WHITE),
                        },
                    )
                    .with_justify(JustifyText::Center),
                    transform: Transform::from_xyz(0.0, HEIGHT / 2. + 7., 1.),
                    ..default()
                },
                CharacterInfo,
            ));

            // The health bar.
            let shape = shapes::Rectangle {
                extents: Vec2::new(WIDTH + 2., 5.),
                ..default()
            };
            children.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(0., HEIGHT / 2. + 1., 1.),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    ..default()
                },
                draw::Fill::color(Color::LinearRgba(LinearRgba::BLACK)),
                CharacterHealthBar,
            ));

            children.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(0., HEIGHT / 2. + 1., 1.1),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    ..default()
                },
                draw::Fill::color(Color::LinearRgba(LinearRgba::RED)),
                CharacterHealthBarInner,
            ));

            // The "grass effect" (invisible for the moment).
            children.spawn(GrassEffectBundle::new(HEIGHT, asset_server));
        });
}

macro_rules! set_vis {
    ($vis:ident, $character:ident, $visibility:ident) => {
        if $vis.is_none() {
            $vis = Some(if $character.stats.health.is_full() {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            });
        }
    };
}

// TODO: move it into `character.rs`?
pub fn update_character_info(
    characters: Query<
        (&Character, &Children),
        (Changed<Character>, Without<crate::player::Player>),
    >,
    mut info: Query<(Entity, &mut Visibility), With<CharacterHealthBar>>,
    mut paths: Query<
        (&mut Path, &mut Transform, &mut Visibility),
        (With<CharacterHealthBarInner>, Without<CharacterHealthBar>),
    >,
) {
    'main: for (character, children) in characters.iter() {
        let mut vis = None;
        for child in children.iter() {
            if let Ok((entity, mut visibility)) = info.get_mut(*child) {
                set_vis!(vis, character, visibility);
                *visibility = vis.unwrap();
            }
            if let Ok((mut path, mut transform, mut visibility)) = paths.get_mut(*child) {
                set_vis!(vis, character, visibility);
                // We only update the size of the red bar.
                let new_width =
                    WIDTH * character.stats.health.value() / character.stats.health.max_value();
                transform.translation.x = -(WIDTH / 2. - new_width / 2.);
                *path = ShapePath::build_as(&shapes::Rectangle {
                    extents: Vec2::new(new_width, 3.),
                    ..default()
                });
                *visibility = vis.unwrap();
            }
        }
    }
}
