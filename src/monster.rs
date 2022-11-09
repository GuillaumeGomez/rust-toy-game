use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::{draw, render};
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterInfo, CharacterPoints,
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
    let texture_atlas =
        TextureAtlas::from_grid(skeleton_texture, Vec2::new(48., 48.), NB_ANIMATIONS, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let level = 2;

    commands
        .spawn()
        .insert(Skeleton)
        .insert(Character::new(
            level,
            0,
            // FIXME, should have a method for a specific level.
            CharacterPoints::level_1(),
            WIDTH,
            HEIGHT,
        ))
        .insert(CharacterAnimationInfo {
            animation_time: ANIMATION_TIME,
            nb_animations: NB_ANIMATIONS,
            timer: Timer::from_seconds(ANIMATION_TIME, true),
            animation_type: CharacterAnimationType::ForwardIdle,
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2 {
                    x: WIDTH,
                    y: HEIGHT,
                }),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(LockedAxes::ROTATION_LOCKED)
        .with_children(|children| {
            // move box
            children
                .spawn()
                .insert(Collider::cuboid(8.0, 7.0))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -5.0, 0.0)))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ));
            children
                .spawn()
                .insert(Collider::cuboid(WIDTH / 2. - 6., HEIGHT / 2. - 1.))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)))
                .insert(Sensor)
                .insert(CollisionGroups::new(crate::HITBOX, crate::HITBOX));

            children
                .spawn_bundle(Text2dBundle {
                    text: Text::from_section(
                        &format!("Skeleton lvl. {}", level),
                        TextStyle {
                            font: asset_server.load(crate::FONT),
                            font_size: 10.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::CENTER),
                    ..default()
                })
                .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    0.0,
                    HEIGHT / 2. + 7.,
                    0.0,
                )))
                .insert(CharacterInfo);

            children
                .spawn_bundle(
                    GeometryBuilder::new()
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
                        ),
                )
                .insert(CharacterInfo)
                .insert(Visibility { is_visible: false });
            children
                .spawn_bundle(
                    GeometryBuilder::new()
                        .add(&shapes::Rectangle {
                            extents: Vec2::new(WIDTH, 2.),
                            ..default()
                        })
                        .build(
                            DrawMode::Fill(draw::FillMode::color(Color::RED)),
                            Transform::from_xyz(0., HEIGHT / 2. + 1., 0.),
                        ),
                )
                .insert(CharacterInfo)
                .insert(Visibility { is_visible: false });
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            200.0, 210.0, 0.0,
        )))
        .insert(OutsideWorld);
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
