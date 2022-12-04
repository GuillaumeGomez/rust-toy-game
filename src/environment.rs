use bevy::ecs::system::EntityCommands;
use bevy::math::Rect;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::geometry::CollisionEventFlags;
use rand::Rng;
use rand_seeder::Seeder;

use crate::character::{Character, GrassEffect};

#[derive(Debug, Component)]
pub struct Bush;

fn insert_bush(texture: Handle<Image>, commands: &mut Commands, x: f32, y: f32) {
    commands
        .spawn((
            Bush,
            crate::game::OutsideWorld,
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            children.spawn((
                Collider::ball(16.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
            ));
        });
}

#[derive(Debug, Component, Clone, Copy)]
#[repr(usize)]
enum Tree {
    Normal = 0,
    Sapin = 1,
    Scary = 2,
    Dead = 3,
}

impl Tree {
    fn add_colliders(self, children: &mut ChildBuilder, group: Group) {
        match self {
            Tree::Normal => {
                children.spawn((Collider::ball(32.), CollisionGroups::new(group, group)));
            }
            Tree::Sapin => {
                children.spawn((Collider::ball(29.), CollisionGroups::new(group, group)));
            }
            Tree::Scary => {
                children.spawn((
                    Collider::ball(29.),
                    CollisionGroups::new(group, group),
                    TransformBundle::from(Transform::from_xyz(0.0, 6.0, 0.0)),
                ));
            }
            Tree::Dead => {
                children.spawn((
                    Collider::cuboid(8., 24.),
                    CollisionGroups::new(group, group),
                    TransformBundle::from(Transform::from_xyz(0.0, 8.0, 0.0)),
                ));
                children.spawn((
                    Collider::cuboid(4., 8.),
                    CollisionGroups::new(group, group),
                    TransformBundle::from(Transform::from_xyz(14.0, 10.0, 0.0)),
                ));
                children.spawn((
                    Collider::cuboid(4., 3.),
                    CollisionGroups::new(group, group),
                    TransformBundle::from(Transform::from_xyz(-14.0, 11.0, 0.0)),
                ));
            }
        }
    }
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

fn insert_tree(
    trees: &[usize],
    texture: Handle<TextureAtlas>,
    commands: &mut Commands,
    tree: Tree,
    x: f32,
    y: f32,
) {
    commands
        .spawn((
            tree,
            crate::game::OutsideWorld,
            SpriteSheetBundle {
                texture_atlas: texture,
                sprite: TextureAtlasSprite {
                    index: tree as _,
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            tree.add_colliders(children, crate::OUTSIDE_WORLD);
        });
}

#[derive(Debug, Component)]
pub struct Grass;

fn insert_grass(
    texture: Handle<TextureAtlas>,
    commands: &mut Commands,
    x: f32,
    y: f32,
    max_row: usize,
    max_col: usize,
) {
    // first we generate the "grid" of grass.
    let mut rng: crate::SeedType =
        Seeder::from(&format!("{};{:.1};{:.1}", crate::SEED, x, y)).make_rng();
    let mut v = Vec::with_capacity(max_row);
    for _ in 0..max_row {
        let mut line = Vec::with_capacity(max_col);
        for _ in 0..max_col {
            // 70% of chance to have grass.
            line.push((rng.gen::<u8>() % 10) > 2);
        }
        v.push(line);
    }

    // Then we generate the texture.
    for (row, line) in v.iter().enumerate() {
        let mut nb = 0;
        for (pos, entry) in line.iter().enumerate() {
            if !*entry {
                continue;
            }
            let has_grass_below = v.get(row + 1).map(|c| c[pos]).unwrap_or(false);
            let index = if has_grass_below { nb % 5 } else { nb % 5 + 5 };
            nb += 1;
            commands.spawn((
                Grass,
                crate::game::OutsideWorld,
                SpriteSheetBundle {
                    texture_atlas: texture.clone(),
                    sprite: TextureAtlasSprite {
                        index,
                        custom_size: Some(Vec2 {
                            x: crate::GRASS_SIZE,
                            y: crate::GRASS_SIZE,
                        }),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        x + pos as f32 * crate::GRASS_SIZE,
                        y - row as f32 * crate::GRASS_SIZE,
                        0.0,
                    ),
                    ..default()
                },
                Sensor,
                Collider::cuboid(crate::GRASS_SIZE / 2., crate::GRASS_SIZE / 2.),
                CollisionGroups::new(
                    crate::OUTSIDE_WORLD | crate::HITBOX,
                    crate::OUTSIDE_WORLD | crate::HITBOX,
                ),
            ));
        }
    }
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
    let trees = vec![
        texture_atlas.add_texture(Rect {
            min: Vec2::new(0., 0.),
            max: Vec2::new(64., 64.),
        }),
        texture_atlas.add_texture(Rect {
            min: Vec2::new(64., 0.),
            max: Vec2::new(128., 64.),
        }),
        texture_atlas.add_texture(Rect {
            min: Vec2::new(128., 0.),
            max: Vec2::new(192., 71.),
        }),
        texture_atlas.add_texture(Rect {
            min: Vec2::new(193., 0.),
            max: Vec2::new(234., 64.),
        }),
    ];
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut x = -100.;
    let mut y = 280.;
    for nb in 0..6 {
        if nb % 3 == 0 {
            x = -100.;
            y += 100.;
        }
        insert_tree(
            &trees,
            texture_atlas_handle.clone(),
            &mut commands,
            (nb % 4).into(),
            x,
            y,
        );
        x += 100.;
    }

    let grass_texture = asset_server.load("textures/grass.png");
    let mut texture_atlas =
        TextureAtlas::from_grid(grass_texture, Vec2::new(16., 16.), 5, 2, None, None);
    let grass_texture_handle = texture_atlases.add(texture_atlas);

    insert_grass(
        grass_texture_handle.clone(),
        &mut commands,
        -450.,
        250.,
        3,
        10,
    );
    insert_grass(
        grass_texture_handle.clone(),
        &mut commands,
        -500.,
        400.,
        3,
        10,
    );
}

fn check_if_grass(
    characters: &Query<&Children, With<Character>>,
    grass: &Query<&Sensor, With<Grass>>,
    grass_effect: &mut Query<(Entity, &mut GrassEffect, &mut Visibility)>,
    x: &Entity,
    y: &Entity,
    count: isize,
) -> bool {
    if !grass.get(*x).is_ok() {
        return false;
    }
    if let Some(children) = characters.iter().find(|children| children.contains(y)) {
        if let Some((_, mut effect, mut visibility)) = grass_effect
            .iter_mut()
            .find(|(e, _, _)| children.contains(e))
        {
            effect.count += count;
            visibility.is_visible = effect.count > 0;
        }
    }
    true
}

pub fn grass_events(
    mut collision_events: EventReader<CollisionEvent>,
    characters: Query<&Children, With<Character>>,
    grass: Query<&Sensor, With<Grass>>,
    mut grass_effect: Query<(Entity, &mut GrassEffect, &mut Visibility)>,
) {
    // First we go through all "stopped collisions".
    for collision_event in collision_events.iter() {
        let (count, x, y) = match collision_event {
            CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) => (1, x, y),
            CollisionEvent::Stopped(x, y, CollisionEventFlags::SENSOR) => (-1, x, y),
            _ => continue,
        };
        if !check_if_grass(&characters, &grass, &mut grass_effect, x, y, count) {
            !check_if_grass(&characters, &grass, &mut grass_effect, y, x, count);
        }
    }
}
