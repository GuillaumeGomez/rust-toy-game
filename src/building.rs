use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct House {
    is_open: bool,
}

fn insert_building(texture: Handle<TextureAtlas>, commands: &mut Commands, x: f32, y: f32, is_open: bool) {
    commands
        .spawn()
        .insert(House { is_open })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture,
            sprite: TextureAtlasSprite {
                index: is_open as _,
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .with_children(|children| {
            children.spawn()
                .insert(Collider::cuboid(40., 35.))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 7.0, 0.0)));
            children.spawn()
                .insert(Collider::cuboid(14., 9.))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -30.0, 0.0)));
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
        let is_open = nb & 1 == 0;
        insert_building(texture_atlas_handle.clone(), &mut commands, -100., (nb as f32) * 150., is_open);
        insert_building(texture_atlas_handle.clone(), &mut commands, -400., (nb as f32) * 150., is_open);
    }
}
