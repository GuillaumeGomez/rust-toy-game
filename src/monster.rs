use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterPoints,
};
use crate::game::OutsideWorld;

#[derive(Component)]
pub struct Skeleton;

pub fn spawn_monsters(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    const NB_ANIMATIONS: usize = 3;
    const ANIMATION_TIME: f32 = 0.15;
    const WIDTH: f32 = 26.;
    const HEIGHT: f32 = 26.;

    let skeleton_texture = asset_server.load("textures/skeleton.png");
    let texture_atlas =
        TextureAtlas::from_grid(skeleton_texture, Vec2::new(48., 48.), NB_ANIMATIONS, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn()
        .insert(Skeleton)
        .insert(Character::new(2, 0, CharacterPoints::level_1()))
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
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            200.0, 210.0, 0.0,
        )))
        .insert(OutsideWorld);
}
