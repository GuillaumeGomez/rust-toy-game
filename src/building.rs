use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterKind, CharacterPoints,
};

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
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

struct CarpetDimension {
    width: u8,
    height: u8,
}

enum CarpetColor {
    Green,
    Red,
    Violet,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub enum Furniture {
    Desk,
    SmallTable,
    LongTable,
    Stool,
    Crate,
    Bed,
    Carpet,
    DoorCarpet,
    MuralSwords,
    MuralTools,
}

impl Furniture {
    fn build_carpet<C: Component>(
        self,
        commands: &mut Commands,
        texture: Handle<Image>,
        x: f32,
        y: f32,
        state: C,
        dimension: CarpetDimension,
        color: CarpetColor,
    ) {
        let (x_image, y_image, corner_width, corner_height, border_width) = match color {
            CarpetColor::Green => (0., 72., 16., 16., 16.),
            CarpetColor::Red => (51., 70., 8., 8., 8.),
            CarpetColor::Violet => (50., 104., 12., 12., 8.),
        };
        let total_width = corner_width * 2. + border_width * dimension.width as f32;
        // let total_height = corner_height * 2. + border_width * dimension.height as f32;
        let mut c = commands
            .spawn((
                state,
                TransformBundle::from_transform(Transform::from_xyz(x, y, self.z_index())),
                VisibilityBundle::default(),
            ))
            .with_children(|children| {
                // top-left corner.
                children.spawn(
                    (SpriteBundle {
                        texture: texture.clone(),
                        sprite: Sprite {
                            rect: Some(Rect::new(
                                x_image,
                                y_image,
                                x_image + corner_width,
                                y_image + corner_height,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(0., 0., 0.),
                        ..default()
                    }),
                );
                // Not sure exactly why this diff is needed for violet carpet...
                let diff = (corner_width - border_width) / 2.;
                // Top border.
                for pos in 0..dimension.width {
                    children.spawn(
                        (SpriteBundle {
                            texture: texture.clone(),
                            sprite: Sprite {
                                rect: Some(Rect::new(
                                    x_image + corner_width,
                                    y_image,
                                    x_image + corner_width + border_width,
                                    y_image + corner_height,
                                )),
                                ..default()
                            },
                            transform: Transform::from_xyz(
                                corner_width - diff + border_width * (pos as f32),
                                0.,
                                0.,
                            ),
                            ..default()
                        }),
                    );
                }
                // top-right corner.
                children.spawn(
                    (SpriteBundle {
                        texture: texture.clone(),
                        sprite: Sprite {
                            rect: Some(Rect::new(
                                x_image + corner_width + border_width,
                                y_image,
                                x_image + corner_width * 2. + border_width,
                                y_image + corner_height,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            corner_width + border_width * (dimension.width as f32),
                            0.,
                            0.,
                        ),
                        ..default()
                    }),
                );
                for pos_y in 0..dimension.height {
                    // left border
                    children.spawn(
                        (SpriteBundle {
                            texture: texture.clone(),
                            sprite: Sprite {
                                rect: Some(Rect::new(
                                    x_image,
                                    y_image + corner_height,
                                    x_image + corner_width,
                                    y_image + corner_height + border_width, // `border_width` because rotated.
                                )),
                                ..default()
                            },
                            transform: Transform::from_xyz(
                                0.,
                                (corner_height - diff + border_width * pos_y as f32) * -1.,
                                0.,
                            ),
                            ..default()
                        }),
                    );
                    for pos_x in 0..dimension.width {
                        // The "middle".
                        children.spawn(
                            (SpriteBundle {
                                texture: texture.clone(),
                                sprite: Sprite {
                                    rect: Some(Rect::new(
                                        x_image + corner_width,
                                        y_image + corner_height,
                                        x_image + corner_width + border_width,
                                        y_image + corner_height + border_width, // `border_width` because rotated.
                                    )),
                                    ..default()
                                },
                                transform: Transform::from_xyz(
                                    corner_width - diff + border_width * (pos_x as f32),
                                    (corner_height - diff + corner_height * pos_y as f32) * -1.,
                                    0.,
                                ),
                                ..default()
                            }),
                        );
                    }
                    // right border
                    children.spawn(
                        (SpriteBundle {
                            texture: texture.clone(),
                            sprite: Sprite {
                                rect: Some(Rect::new(
                                    x_image + corner_width + border_width,
                                    y_image + corner_height,
                                    x_image + corner_width + border_width + corner_width,
                                    y_image + corner_height + border_width, // `border_width` because rotated.
                                )),
                                ..default()
                            },
                            transform: Transform::from_xyz(
                                corner_width + border_width * dimension.width as f32,
                                (corner_height - diff + border_width * pos_y as f32) * -1.,
                                0.,
                            ),
                            ..default()
                        }),
                    );
                }
                // bottom-left corner.
                children.spawn(
                    (SpriteBundle {
                        texture: texture.clone(),
                        sprite: Sprite {
                            rect: Some(Rect::new(
                                x_image,
                                y_image + corner_height + border_width,
                                x_image + corner_width,
                                y_image + corner_height * 2. + border_width,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            0.,
                            (corner_height + border_width * dimension.height as f32) * -1.,
                            0.,
                        ),
                        ..default()
                    }),
                );
                // bottom border.
                for pos in 0..dimension.width {
                    children.spawn(
                        (SpriteBundle {
                            texture: texture.clone(),
                            sprite: Sprite {
                                rect: Some(Rect::new(
                                    x_image + corner_width,
                                    y_image + corner_height + border_width,
                                    x_image + corner_width + border_width,
                                    y_image + corner_height + border_width + corner_height,
                                )),
                                ..default()
                            },
                            transform: Transform::from_xyz(
                                corner_width - diff + border_width * (pos as f32),
                                (corner_height + border_width * dimension.height as f32) * -1.,
                                0.,
                            ),
                            ..default()
                        }),
                    );
                }
                // bottom-right corner.
                children.spawn(
                    (SpriteBundle {
                        texture: texture,
                        sprite: Sprite {
                            rect: Some(Rect::new(
                                x_image + corner_width + border_width,
                                y_image + corner_height + border_width,
                                x_image + corner_width * 2. + border_width,
                                y_image + corner_height * 2. + border_width,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            corner_width + border_width * (dimension.width as f32),
                            (corner_height + border_width * dimension.height as f32) * -1.,
                            0.,
                        ),
                        ..default()
                    }),
                );
            });
    }

    fn pos_in_image(self) -> Rect {
        match self {
            Self::Desk => Rect::new(0., 0., 80., 24.),
            Self::SmallTable => Rect::new(220., 0., 260., 30.),
            Self::LongTable => Rect::new(219., 227., 267., 63.),
            Self::Stool => Rect::new(132., 0., 148., 16.),
            Self::Crate => Rect::new(132., 17., 148., 39.),
            Self::Bed => Rect::new(183., 0., 215., 48.),
            Self::DoorCarpet => Rect::new(0., 121., 48., 136.),
            Self::Carpet => panic!("should use `build_carpet!`"),
            Self::MuralSwords => Rect::new(150., 0., 164., 33.),
            Self::MuralTools => Rect::new(167., 0., 180., 29.),
        }
    }

    fn get_collider(self) -> Option<(f32, f32)> {
        const TOP_SIZE: f32 = 3.;

        match self {
            Self::Carpet | Self::DoorCarpet | Self::MuralSwords | Self::MuralTools => None,
            _ => {
                let size = self.pos_in_image();
                Some((size.width() / 2., size.height() / 2. - TOP_SIZE))
            }
        }
    }

    fn z_index(self) -> f32 {
        match self {
            Self::Carpet | Self::DoorCarpet => crate::CARPET_Z_INDEX,
            _ => crate::FURNITURE_Z_INDEX,
        }
    }
}

#[derive(Debug, Component)]
pub struct Door;
#[derive(Debug, Component)]
pub struct EnterArea;

fn insert_furniture<C: Component>(
    commands: &mut Commands,
    furnitures_texture: Handle<Image>,
    furniture: Furniture,
    x: f32,
    y: f32,
    state: C,
    flip: bool,
) {
    if let Some((collider_width, collider_height)) = furniture.get_collider() {
        let mut img = furniture.pos_in_image();
        let mut img_bottom = img;
        let height = img.height() / 2.;
        img_bottom.min.y += height;
        commands
            .spawn((
                state,
                SpriteBundle {
                    texture: furnitures_texture.clone(),
                    sprite: Sprite {
                        rect: Some(img_bottom),
                        flip_x: flip,
                        flip_y: flip,
                        ..default()
                    },
                    transform: Transform::from_xyz(x, y, furniture.z_index()),
                    ..default()
                },
            ))
            .with_children(|children| {
                img.max.y -= height;
                children.spawn(
                    (SpriteBundle {
                        texture: furnitures_texture,
                        sprite: Sprite {
                            rect: Some(img),
                            flip_x: flip,
                            flip_y: flip,
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            0.,
                            height,
                            crate::FURNITURE_TOP_PART_Z_INDEX,
                        ),
                        ..default()
                    }),
                );
                // We need to create the collider here so we can give it the right position.
                children.spawn((
                    RigidBody::Fixed,
                    Collider::cuboid(collider_width, collider_height),
                    CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                    TransformBundle::from(Transform::from_xyz(0., height - height / 2., 0.)),
                ));
            });
    } else {
        commands.spawn((
            state,
            SpriteBundle {
                texture: furnitures_texture,
                sprite: Sprite {
                    rect: Some(furniture.pos_in_image()),
                    flip_x: flip,
                    flip_y: flip,
                    ..default()
                },
                transform: Transform::from_xyz(x, y, furniture.z_index()),
                ..default()
            },
        ));
    }
}

const GENERAL_SHOP_HEIGHT: u32 = 106;
const GENERAL_SHOP_WIDTH: u32 = 110;
const GENERAL_SHOP_HEIGHT_F: f32 = GENERAL_SHOP_HEIGHT as f32;
const GENERAL_SHOP_WIDTH_F: f32 = GENERAL_SHOP_WIDTH as f32;
const WEAPON_SHOP_HEIGHT: u32 = 106;
const WEAPON_SHOP_WIDTH: u32 = 110;

fn insert_shop(
    layout: Handle<TextureAtlasLayout>,
    texture: Handle<Image>,
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
            SpriteBundle {
                texture: texture.clone(),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2 {
                        x: GENERAL_SHOP_WIDTH_F,
                        y: GENERAL_SHOP_HEIGHT_F,
                    }),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: layout.clone(),
                index: start_index + 1,
            },
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            children.spawn((
                TextureAtlas {
                    layout,
                    index: start_index,
                },
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: GENERAL_SHOP_WIDTH_F,
                            y: GENERAL_SHOP_HEIGHT_F,
                        }),
                        ..default()
                    },
                    texture,
                    transform: Transform::from_xyz(0., 0., crate::BUILDING_TOP_PART_Z_INDEX),
                    ..default()
                },
            ));
            // The roof.
            children.spawn((
                Collider::cuboid(48., 28.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
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

fn insert_house(
    layout: Handle<TextureAtlasLayout>,
    texture: Handle<Image>,
    commands: &mut Commands,
    x: f32,
    y: f32,
) {
    const TOP_SIZE: f32 = 60.;
    const BOTTOM_SIZE: f32 = 88. - TOP_SIZE;
    const WIDTH: f32 = 80.;

    commands
        .spawn((
            Building::House,
            crate::game::OutsideWorld,
            TextureAtlas {
                index: false as _, // door open is false so index is 0
                layout: layout.clone(),
            },
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2 {
                        x: 80.,
                        y: BOTTOM_SIZE,
                    }),
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_xyz(x, y - TOP_SIZE, 0.0),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            const Y: f32 = TOP_SIZE / 2.;
            children.spawn((
                TextureAtlas { index: 2, layout },
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: 80.,
                            y: TOP_SIZE,
                        }),
                        ..default()
                    },
                    texture,
                    transform: Transform::from_xyz(
                        0.,
                        BOTTOM_SIZE / 2. + TOP_SIZE / 2.,
                        crate::BUILDING_TOP_PART_Z_INDEX,
                    ),
                    ..default()
                },
            ));
            children.spawn((
                Collider::cuboid(39., 25.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(0.0, Y, 0.0)),
            ));
            // The porch (left).
            children.spawn((
                Collider::cuboid(2., 8.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(14.0, Y - 30.0, 0.0)),
            ));
            // The porch (right).
            children.spawn((
                Collider::cuboid(2., 8.),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                TransformBundle::from(Transform::from_xyz(-14.0, Y - 30.0, 0.0)),
            ));
            // The "enter area" sensor.
            children.spawn((
                Collider::cuboid(0.5, 8.),
                EnterArea,
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                Sensor,
                TransformBundle::from(Transform::from_xyz(0.0, Y - 20.0, 0.0)),
            ));
            // The door sensor.
            children.spawn((
                Collider::cuboid(14., 11.),
                Door,
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                Sensor,
                TransformBundle::from(Transform::from_xyz(0.0, Y - 36.0, 0.0)),
            ));
        });
}

pub fn spawn_buildings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let house_texture = asset_server.load("textures/house.png");
    let mut house_texture_atlas = TextureAtlasLayout::new_empty(UVec2::new(160, 88));
    // We split the top from the bottom part.
    // First we add both lower parts (open and closed door).
    house_texture_atlas.add_texture(URect::new(0, 61, 80, 88));
    house_texture_atlas.add_texture(URect::new(80, 61, 160, 88));
    // Then we add the upper part.
    house_texture_atlas.add_texture(URect::new(0, 0, 80, 60));

    let house_texture_atlas_handle = texture_atlases.add(house_texture_atlas);

    for nb in 0..2 {
        insert_house(
            house_texture_atlas_handle.clone(),
            house_texture.clone(),
            &mut commands,
            -100.,
            (nb as f32) * 150.,
        );
        insert_house(
            house_texture_atlas_handle.clone(),
            house_texture.clone(),
            &mut commands,
            -400.,
            (nb as f32) * 150.,
        );
    }

    let shops_texture = asset_server.load("textures/shops.png");
    let shops_texture_atlas = TextureAtlasLayout::from_grid(
        UVec2::new(GENERAL_SHOP_WIDTH, GENERAL_SHOP_HEIGHT),
        2,
        2,
        None,
        None,
    );
    let shops_texture_atlas_handle = texture_atlases.add(shops_texture_atlas);
    insert_shop(
        shops_texture_atlas_handle.clone(),
        shops_texture.clone(),
        &mut commands,
        Building::GeneralShop,
        -220.,
        270.,
    );
    insert_shop(
        shops_texture_atlas_handle,
        shops_texture,
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
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let building = app_state.building.unwrap();
    let house_texture = match building {
        Building::House => asset_server.load("textures/inside-house.png"),
        Building::WeaponShop | Building::GeneralShop => {
            asset_server.load("textures/inside-shop.png")
        }
    };

    let x = app_state.pos.x;
    let y = app_state.pos.y;

    let width = 237.;
    let height = 160.;

    commands
        .spawn((
            building,
            SpriteBundle {
                texture: house_texture,
                transform: Transform::from_xyz(x, y, crate::BACKGROUND_Z_INDEX),
                sprite: Sprite {
                    custom_size: Some(Vec2 {
                        x: width,
                        y: height,
                    }),
                    ..default()
                },
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

    let furnitures_texture = asset_server.load("textures/furnitures.png");

    if matches!(building, Building::WeaponShop | Building::GeneralShop) {
        let desk_y = y + 5.;
        insert_furniture(
            &mut commands,
            furnitures_texture.clone(),
            Furniture::Desk,
            x,
            y + 5.,
            crate::game::InsideHouse,
            false,
        );
        let is_weapon_shop = building == Building::WeaponShop;
        if is_weapon_shop {
            let dim = Furniture::MuralSwords.pos_in_image();
            insert_furniture(
                &mut commands,
                furnitures_texture.clone(),
                Furniture::MuralSwords,
                x + width / 2. - dim.width() - 2.,
                y + height / 4. - dim.height(),
                crate::game::InsideHouse,
                false,
            );
            insert_furniture(
                &mut commands,
                furnitures_texture.clone(),
                Furniture::MuralSwords,
                x - width / 2. + dim.width() - 2.,
                y + height / 4. - dim.height(),
                crate::game::InsideHouse,
                true,
            );
        }
        let pos_in_image = Furniture::Desk.pos_in_image();
        crate::vendor::spawn_vendor(
            &mut commands,
            asset_server,
            texture_atlases,
            x,
            desk_y + pos_in_image.height(),
            crate::game::InsideHouse,
            is_weapon_shop,
        );
    } else {
        insert_furniture(
            &mut commands,
            furnitures_texture.clone(),
            Furniture::Bed,
            x - 73.,
            y - 20.,
            crate::game::InsideHouse,
            false,
        );
        insert_furniture(
            &mut commands,
            furnitures_texture.clone(),
            Furniture::DoorCarpet,
            x,
            y - 41.,
            crate::game::InsideHouse,
            false,
        );
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub enum Statue {
    Magus = 0,
    Knight = 1,
    Archer = 2,
}

impl Statue {
    const HEIGHT: f32 = 96.;

    fn create(
        &self,
        commands: &mut Commands,
        atlas: Handle<TextureAtlasLayout>,
        texture: Handle<Image>,
        x: f32,
        y: f32,
    ) {
        let index = *self as usize;
        let (width, offset_x) = if index == Statue::Magus as usize {
            (54., -2.)
        } else {
            (46., 0.)
        };
        commands
            .spawn((
                *self,
                crate::game::OutsideWorld,
                TextureAtlas {
                    layout: atlas.clone(),
                    index: index * 2,
                },
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 { x: width, y: 59. }),
                        ..default()
                    },
                    texture: texture.clone(),
                    transform: Transform::from_xyz(x, y, crate::BUILDING_TOP_PART_Z_INDEX),
                    ..default()
                },
                RigidBody::Fixed,
            ))
            .with_children(|children| {
                children.spawn((
                    TextureAtlas {
                        layout: atlas,
                        index: index * 2 + 1,
                    },
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2 {
                                x: width,
                                y: Self::HEIGHT - 60.,
                            }),
                            ..default()
                        },
                        texture: texture.clone(),
                        transform: Transform::from_xyz(
                            0.,
                            -47.,
                            crate::FURNITURE_Z_INDEX - crate::BUILDING_TOP_PART_Z_INDEX,
                        ),
                        ..default()
                    },
                ));
                children.spawn((
                    Collider::cuboid(46. / 2., 46. / 2.),
                    CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                    TransformBundle::from(Transform::from_xyz(offset_x, -42., 0.)),
                ));
            });
    }
}

pub fn spawn_statues(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let statues_texture = asset_server.load("textures/statues.png");
    let mut statues_texture_atlas = TextureAtlasLayout::new_empty(UVec2::new(153, 96));
    // We split the top from the bottom part.
    statues_texture_atlas.add_texture(URect::new(0, 0, 54, 59));
    statues_texture_atlas.add_texture(URect::new(0, 60, 54, 96));
    statues_texture_atlas.add_texture(URect::new(57, 0, 103, 59));
    statues_texture_atlas.add_texture(URect::new(57, 60, 103, 96));
    statues_texture_atlas.add_texture(URect::new(107, 0, 153, 59));
    statues_texture_atlas.add_texture(URect::new(107, 60, 153, 96));
    let statues_texture_atlas_handle = texture_atlases.add(statues_texture_atlas);

    Statue::Magus.create(
        &mut commands,
        statues_texture_atlas_handle.clone(),
        statues_texture.clone(),
        0.,
        0.,
    );
    Statue::Knight.create(
        &mut commands,
        statues_texture_atlas_handle.clone(),
        statues_texture.clone(),
        70.,
        0.,
    );
    Statue::Archer.create(
        &mut commands,
        statues_texture_atlas_handle.clone(),
        statues_texture.clone(),
        140.,
        0.,
    );
}
