use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component)]
pub struct House;

#[derive(Debug, Component)]
pub struct Door;
#[derive(Debug, Component)]
pub struct EnterArea;

fn insert_building(texture: Handle<TextureAtlas>, commands: &mut Commands, x: f32, y: f32) {
    commands
        .spawn()
        .insert(House)
        .insert(crate::game::OutsideWorld)
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture,
            sprite: TextureAtlasSprite {
                index: false as _,
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .with_children(|children| {
            children
                .spawn()
                .insert(Collider::cuboid(40., 35.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 7.0, 0.0)));
            // The porch (left).
            children
                .spawn()
                .insert(Collider::cuboid(2., 8.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(14.0, -30.0, 0.0)));
            // The porch (right).
            children
                .spawn()
                .insert(Collider::cuboid(2., 8.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    -14.0, -30.0, 0.0,
                )));
            // The "enter area" sensor.
            children
                .spawn()
                .insert(Collider::cuboid(0.5, 8.))
                .insert(EnterArea)
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert(Sensor)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -20.0, 0.0)));
            // The door sensor.
            children
                .spawn()
                .insert(Collider::cuboid(14., 9.))
                .insert(Door)
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert(Sensor)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -36.0, 0.0)));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(x, y, 0.0)));
}

pub fn spawn_buildings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let house_texture = asset_server.load("textures/house.png");
    let texture_atlas = TextureAtlas::from_grid(house_texture, Vec2::new(80., 88.), 2, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for nb in 0..2 {
        insert_building(
            texture_atlas_handle.clone(),
            &mut commands,
            -100.,
            (nb as f32) * 150.,
        );
        insert_building(
            texture_atlas_handle.clone(),
            &mut commands,
            -400.,
            (nb as f32) * 150.,
        );
    }
}

pub fn spawn_inside_building(mut commands: Commands, asset_server: Res<AssetServer>) {
    let house_texture = asset_server.load("textures/inside-house.png");

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            texture: house_texture,
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(crate::game::InsideHouse)
        .with_children(|children| {
            // The left and right walls.
            children
                .spawn()
                .insert(Collider::cuboid(16., 57.))
                .insert(CollisionGroups::new(
                    crate::NOT_OUTSIDE_WORLD,
                    crate::NOT_OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(-106.0, 0.0, 0.0)));
            children
                .spawn()
                .insert(Collider::cuboid(16., 57.))
                .insert(CollisionGroups::new(
                    crate::NOT_OUTSIDE_WORLD,
                    crate::NOT_OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(106.0, 0.0, 0.0)));
            // The top wall.
            children
                .spawn()
                .insert(Collider::cuboid(92., 16.))
                .insert(CollisionGroups::new(
                    crate::NOT_OUTSIDE_WORLD,
                    crate::NOT_OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 66.0, 0.0)));
            // The bottom wall (left part).
            children
                .spawn()
                .insert(Collider::cuboid(55., 16.))
                .insert(CollisionGroups::new(
                    crate::NOT_OUTSIDE_WORLD,
                    crate::NOT_OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    -68.0, -65.0, 0.0,
                )));
            // The bottom wall (right part).
            children
                .spawn()
                .insert(Collider::cuboid(55., 16.))
                .insert(CollisionGroups::new(
                    crate::NOT_OUTSIDE_WORLD,
                    crate::NOT_OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(68.0, -65.0, 0.0)));
            // The exit door.
            children
                .spawn()
                .insert(Collider::cuboid(8., 4.))
                .insert(CollisionGroups::new(
                    crate::NOT_OUTSIDE_WORLD,
                    crate::NOT_OUTSIDE_WORLD,
                ))
                .insert(Sensor)
                .insert(EnterArea)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0., -70.0, 0.0)));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0., 0., 0.0)));
}
