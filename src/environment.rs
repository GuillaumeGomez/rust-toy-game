use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component)]
pub struct Bush;

fn insert_bush(texture: Handle<Image>, commands: &mut Commands, x: f32, y: f32) {
    println!("inserting bush: ({}, {})", x, y);
    commands
        .spawn()
        .insert(Bush)
        .insert(crate::game::OutsideWorld)
        .insert_bundle(SpriteBundle {
            texture,
            ..default()
        })
        .insert(RigidBody::Fixed)
        .with_children(|children| {
            children
                .spawn()
                .insert(Collider::cuboid(16., 10.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ));
            children
                .spawn()
                .insert(Collider::cuboid(10., 16.))
                .insert(CollisionGroups::new(
                    crate::OUTSIDE_WORLD,
                    crate::OUTSIDE_WORLD,
                ));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(x, y, 0.0)));
}

pub fn spawn_nature(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let bush_texture = asset_server.load("textures/bush.png");

    for nb in 0..8 {
        insert_bush(
            bush_texture.clone(),
            &mut commands,
            nb as f32 * 40.,
            (nb as f32) * 40. + 100.,
        );
    }
}
