use bevy::prelude::*;
use bevy_prototype_lyon::draw;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterBundle, CharacterKind,
    CharacterPoints,
};
use crate::inventory::Inventory;

#[derive(Component)]
struct Vendor;

pub fn spawn_vendor<C: Component>(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    x: f32,
    y: f32,
    state: C,
    is_weapon_vendor: bool,
) {
    const NB_ANIMATIONS: usize = 8;
    const ANIMATION_TIME: f32 = 0.12;
    const HEIGHT: u32 = 29;
    const WIDTH: u32 = 31;
    const HEIGHT_F: f32 = 29.;
    const WIDTH_F: f32 = 31.;

    let vendor_texture = asset_server.load("textures/vendor.png");
    let vendor_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(WIDTH, HEIGHT), NB_ANIMATIONS as _, 2, None, None);
    let vendor_texture_atlas_handle = texture_atlases.add(vendor_texture_atlas);
    let start_index = if is_weapon_vendor { 0 } else { NB_ANIMATIONS };

    commands
        .spawn((
            Vendor,
            state,
            CharacterBundle::new(
                Character::new(
                    1,
                    0,
                    CharacterPoints::level_1(),
                    WIDTH_F,
                    HEIGHT_F,
                    CharacterKind::Human,
                ),
                CharacterAnimationInfo::new_once_with_start_index(
                    ANIMATION_TIME,
                    NB_ANIMATIONS,
                    CharacterAnimationType::ForwardMove,
                    start_index,
                ),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: WIDTH_F,
                            y: HEIGHT_F,
                        }),
                        ..default()
                    },
                    texture: vendor_texture,
                    transform: Transform::from_xyz(
                        x,
                        y + 4.,
                        crate::FURNITURE_TOP_PART_Z_INDEX + 0.2,
                    ),
                    ..default()
                },
                TextureAtlas {
                    index: start_index,
                    layout: vendor_texture_atlas_handle,
                },
                Inventory {
                    // FIXME: Generate a list of items depending of the location of the vendor.
                    items: Vec::new(),
                    gold: 0,
                },
            ),
            RigidBody::Fixed,
        ))
        .with_children(|children| {
            // The "move" box.
            children.spawn((
                Collider::cuboid(8.0, 7.0),
                TransformBundle::from(Transform::from_xyz(0.0, -5.0, 0.0)),
                CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
            ));
            // The "interaction" hitbox.
            children.spawn((
                crate::character::Interaction,
                Collider::cuboid(WIDTH_F / 2., HEIGHT_F / 2. + 2.),
                TransformBundle::from(Transform::from_xyz(0., -4., 0.)),
                Sensor,
                CollisionGroups::new(crate::INTERACTION, crate::INTERACTION),
            ));
        });
}
