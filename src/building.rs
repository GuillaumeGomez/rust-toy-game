use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component)]
pub struct House {
    pub is_open: bool,
    pub contact_with_sensor: u8,
}

#[derive(Debug, Component)]
pub struct InsideHouse;

fn insert_building(texture: Handle<TextureAtlas>, commands: &mut Commands, x: f32, y: f32) {
    commands
        .spawn()
        .insert(House {
            is_open: false,
            contact_with_sensor: 0,
        })
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
            children
                .spawn()
                .insert(Collider::cuboid(14., 8.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -26.0, 0.0)));
            // Same but as a sensor.
            children
                .spawn()
                .insert(Collider::cuboid(0.5, 8.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert(Sensor)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -26.0, 0.0)));
            children
                .spawn()
                .insert(Collider::cuboid(12., 9.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ))
                .insert(Sensor)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -38.0, 0.0)));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(x, y, 0.0)))
        .insert(crate::OutsideWorld);
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

pub fn spawn_inside_building(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let house_texture = asset_server.load("textures/inside-house.png");

    commands
        .spawn()
        .insert(InsideHouse)
        .insert_bundle(SpriteBundle {
            texture: house_texture,
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(crate::NotOutsideWorld)
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
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0., -70.0, 0.0)));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0., 0., 0.0)));
}
