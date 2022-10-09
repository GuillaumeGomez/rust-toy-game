use bevy::prelude::*;
use bevy::sprite::Rect;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component)]
pub struct Bush;

fn insert_bush(texture: Handle<Image>, commands: &mut Commands, x: f32, y: f32) {
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

#[derive(Debug, Component, Clone, Copy)]
#[repr(usize)]
enum Tree {
    Normal = 0,
    Sapin = 1,
    Scary = 2,
    Dead = 3,
}

impl From<usize> for Tree {
    fn from(nb: usize) -> Self {
        match nb {
            0 => Self::Normal,
            1 => Self::Sapin,
            2 => Self::Scary,
            3 => Self::Dead,
            _ => panic!("unexpected number for Tree"),
        }
    }
}

fn insert_tree(trees: &[usize], texture: Handle<TextureAtlas>, commands: &mut Commands, tree: Tree, x: f32, y: f32) {
    commands
        .spawn()
        .insert(tree)
        .insert(crate::game::OutsideWorld)
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture,
            sprite: TextureAtlasSprite {
                index: tree as _,
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .with_children(|children| {
            let (width, height) = match tree {
                Tree::Normal => (32., 32.),
                Tree::Sapin => (32., 32.),
                Tree::Scary => (32., 35.5),
                Tree::Dead => (21., 32.),
            };
            children
                .spawn()
                .insert(Collider::cuboid(width, height))
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

    let trees_texture = asset_server.load("textures/trees.png");
    let mut texture_atlas = TextureAtlas::new_empty(trees_texture, Vec2::new(234., 71.));
    let mut trees = Vec::with_capacity(4);
    trees.push(texture_atlas.add_texture(Rect { min: Vec2::new(0., 0.), max: Vec2::new(64., 64.) }));
    trees.push(texture_atlas.add_texture(Rect { min: Vec2::new(64., 0.), max: Vec2::new(128., 64.) }));
    trees.push(texture_atlas.add_texture(Rect { min: Vec2::new(128., 0.), max: Vec2::new(192., 71.) }));
    trees.push(texture_atlas.add_texture(Rect { min: Vec2::new(192., 0.), max: Vec2::new(234., 64.) }));
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut x = -100.;
    let mut y = 200.;
    for nb in 0..6 {
        if nb % 3 == 0 {
            x = -100.;
            y += 100.;
        }
        insert_tree(&trees, texture_atlas_handle.clone(), &mut commands, (nb % 4).into(), x, y);
        x += 100.;
    }
}
