use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::{draw, render};
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterInfo, CharacterKind,
    CharacterPoints, GrassEffectBundle,
};
use crate::game::OutsideWorld;

#[derive(Component)]
pub struct Skeleton;

const WIDTH: f32 = 26.;
const HEIGHT: f32 = 26.;

pub fn spawn_monsters(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    const NB_ANIMATIONS: usize = 3;
    const ANIMATION_TIME: f32 = 0.15;

    let skeleton_texture = asset_server.load("textures/skeleton.png");
    let texture_atlas = TextureAtlas::from_grid(
        skeleton_texture,
        Vec2::new(48., 48.),
        NB_ANIMATIONS,
        4,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let level = 2;

    commands
        .spawn((
            Skeleton,
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
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite {
                    custom_size: Some(Vec2 {
                        x: WIDTH,
                        y: HEIGHT,
                    }),
                    ..default()
                },
                transform: Transform::from_xyz(200.0, 210.0, 0.0),
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
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::CENTER),
                    transform: Transform::from_xyz(0.0, HEIGHT / 2. + 7., 1.0),
                    ..default()
                },
                CharacterInfo,
            ));

            let mut geometry = GeometryBuilder::new()
                .add(&shapes::Rectangle {
                    extents: Vec2::new(WIDTH, 2.),
                    ..default()
                })
                .build(
                    DrawMode::Outlined {
                        fill_mode: draw::FillMode::color(Color::BLACK),
                        outline_mode: draw::StrokeMode::new(Color::BLACK, 1.5),
                    },
                    Transform::from_xyz(0., HEIGHT / 2. + 1., 0.),
                );
            children.spawn((geometry, CharacterInfo));

            let mut geometry = GeometryBuilder::new()
                .add(&shapes::Rectangle {
                    extents: Vec2::new(WIDTH, 2.),
                    ..default()
                })
                .build(
                    DrawMode::Fill(draw::FillMode::color(Color::RED)),
                    Transform::from_xyz(0., HEIGHT / 2. + 1., 0.),
                );
            geometry.visibility = Visibility { is_visible: false };
            children.spawn((geometry, CharacterInfo));

            // The "grass effect" (invisible for the moment).
            children.spawn(GrassEffectBundle::new(HEIGHT, asset_server));
        });
}

// TODO: move it into `character.rs`?
pub fn update_character_info(
    characters: Query<
        (&Character, &Children),
        (Changed<Character>, Without<crate::player::Player>),
    >,
    mut info: Query<
        (Entity, &mut Visibility, &DrawMode),
        (With<render::Shape>, With<CharacterInfo>),
    >,
    mut paths: Query<(&mut Path, &mut Transform)>,
) {
    for (character, children) in characters.iter() {
        for child in children.iter() {
            if let Ok((entity, mut visibility, draw_mode)) = info.get_mut(*child) {
                visibility.is_visible = !character.stats.health.is_full();
                if !matches!(draw_mode, DrawMode::Fill(_)) {
                    continue;
                }
                if let Ok((mut path, mut transform)) = paths.get_mut(entity) {
                    let new_width =
                        WIDTH * character.stats.health.value() / character.stats.health.max_value();
                    transform.translation.x = -(WIDTH / 2. - new_width / 2.);
                    *path = ShapePath::build_as(&shapes::Rectangle {
                        extents: Vec2::new(new_width, 4.),
                        ..default()
                    });
                }
            }
        }
    }
}
