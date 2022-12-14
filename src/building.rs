use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component)]
pub enum Building {
    House,
    GeneralShop,
    WeaponShop,
}

impl Building {
    fn get_start_index(&self) -> usize {
        match *self {
            Self::House => unreachable!(),
            Self::GeneralShop => 0,
            Self::WeaponShop => 2,
        }
    }
}

#[derive(Debug, Component)]
pub struct Door;
#[derive(Debug, Component)]
pub struct EnterArea;

const GENERAL_SHOP_HEIGHT: f32 = 106.;
const GENERAL_SHOP_WIDTH: f32 = 110.;
const WEAPON_SHOP_HEIGHT: f32 = 106.;
const WEAPON_SHOP_WIDTH: f32 = 110.;

fn insert_shop(
    texture: Handle<TextureAtlas>,
    commands: &mut Commands,
    building: Building,
    x: f32,
    y: f32,
) {
    let start_index = building.get_start_index();

    commands
        .spawn((
            building,
            crate::game::OutsideWorld,
            SpriteSheetBundle {
                texture_atlas: texture.clone(),
                sprite: TextureAtlasSprite {
                    index: start_index + 1,
                    custom_size: Some(Vec2 {
                        x: GENERAL_SHOP_WIDTH,
                        y: GENERAL_SHOP_HEIGHT,
                    }),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            children.spawn((SpriteSheetBundle {
                texture_atlas: texture,
                sprite: TextureAtlasSprite {
                    index: start_index,
                    custom_size: Some(Vec2 {
                        x: GENERAL_SHOP_WIDTH,
                        y: GENERAL_SHOP_HEIGHT,
                    }),
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 2.),
                ..default()
            },));
            // The roof.
            children.spawn((
                Collider::cuboid(50., 38.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(0.0, 8.0, 0.0)),
            ));
            // The porch (left).
            children.spawn((
                Collider::cuboid(15., 8.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(26.0, -40.0, 0.0)),
            ));
            // The porch (right).
            children.spawn((
                Collider::cuboid(15., 8.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(-26.0, -40.0, 0.0)),
            ));
            // The "enter area" sensor.
            children.spawn((
                Collider::cuboid(0.5, 8.),
                EnterArea,
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                Sensor,
                TransformBundle::from(Transform::from_xyz(0.0, -34.0, 0.0)),
            ));
        });
}

fn insert_house(texture: Handle<TextureAtlas>, commands: &mut Commands, x: f32, y: f32) {
    commands
        .spawn((
            Building::House,
            crate::game::OutsideWorld,
            SpriteSheetBundle {
                texture_atlas: texture,
                sprite: TextureAtlasSprite {
                    index: false as _, // door open is false so index is 0
                    custom_size: Some(Vec2 { x: 80., y: 88. }),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            children.spawn((
                Collider::cuboid(40., 35.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(0.0, 7.0, 0.0)),
            ));
            // The porch (left).
            children.spawn((
                Collider::cuboid(2., 8.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(14.0, -30.0, 0.0)),
            ));
            // The porch (right).
            children.spawn((
                Collider::cuboid(2., 8.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(-14.0, -30.0, 0.0)),
            ));
            // The "enter area" sensor.
            children.spawn((
                Collider::cuboid(0.5, 8.),
                EnterArea,
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                Sensor,
                TransformBundle::from(Transform::from_xyz(0.0, -20.0, 0.0)),
            ));
            // The door sensor.
            children.spawn((
                Collider::cuboid(14., 11.),
                Door,
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                Sensor,
                TransformBundle::from(Transform::from_xyz(0.0, -36.0, 0.0)),
            ));
        });
}

pub fn spawn_buildings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let house_texture = asset_server.load("textures/house.png");
    let house_texture_atlas =
        TextureAtlas::from_grid(house_texture, Vec2::new(80., 88.), 2, 1, None, None);
    let house_texture_atlas_handle = texture_atlases.add(house_texture_atlas);

    for nb in 0..2 {
        insert_house(
            house_texture_atlas_handle.clone(),
            &mut commands,
            -100.,
            (nb as f32) * 150.,
        );
        insert_house(
            house_texture_atlas_handle.clone(),
            &mut commands,
            -400.,
            (nb as f32) * 150.,
        );
    }

    let shops_texture = asset_server.load("textures/shops.png");
    let shops_texture_atlas = TextureAtlas::from_grid(
        shops_texture,
        Vec2::new(GENERAL_SHOP_WIDTH, GENERAL_SHOP_HEIGHT),
        2,
        2,
        None,
        None,
    );
    let shops_texture_atlas_handle = texture_atlases.add(shops_texture_atlas);
    insert_shop(
        shops_texture_atlas_handle.clone(),
        &mut commands,
        Building::GeneralShop,
        -220.,
        270.,
    );
    insert_shop(
        shops_texture_atlas_handle,
        &mut commands,
        Building::WeaponShop,
        -100.,
        270.,
    );
}

pub fn spawn_inside_building(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Res<crate::GameInfo>,
) {
    let house_texture = asset_server.load("textures/inside-house.png");

    commands
        .spawn((
            SpriteBundle {
                texture: house_texture,
                transform: Transform::from_xyz(app_state.pos.x, app_state.pos.y, 0.0),
                ..default()
            },
            RigidBody::Fixed,
            crate::game::InsideHouse,
        ))
        .with_children(|children| {
            // The left and right walls.
            children.spawn((
                Collider::cuboid(16., 57.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(-106.0, 0.0, 0.0)),
            ));
            children.spawn((
                Collider::cuboid(16., 57.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(106.0, 0.0, 0.0)),
            ));
            // The top wall.
            children.spawn((
                Collider::cuboid(92., 16.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(0.0, 66.0, 0.0)),
            ));
            // The bottom wall (left part).
            children.spawn((
                Collider::cuboid(55., 16.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(-68.0, -65.0, 0.0)),
            ));
            // The bottom wall (right part).
            children.spawn((
                Collider::cuboid(55., 16.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(68.0, -65.0, 0.0)),
            ));
            // The exit door.
            children.spawn((
                Collider::cuboid(8., 4.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                Sensor,
                EnterArea,
                TransformBundle::from(Transform::from_xyz(0., -70.0, 0.0)),
            ));
        });
}
