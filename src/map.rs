use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::Player;
use crate::MAP_SIZE;

// The first map is at (0, 0). Then every time we go (MAP_SIZE / 2) to a given direction, we
// increase `x` or `xy` (or both) by 1.
pub fn create_map_for_pos(mut commands: Commands, x: u32, y: u32) {
    // commands
    //     .spawn_bundle(TransformBundle::from(Transform::from_xyz(
    //         x as f32 * MAP_SIZE - MAP_SIZE / 2.,
    //         y as f32 * MAP_SIZE - MAP_SIZE / 2.,
    //         0.0,
    //     )))
    //     .insert(Collider::cuboid(MAP_SIZE, MAP_SIZE));
    // TODO: spawn environment.
}

pub fn spawn_map(mut commands: Commands) {
    create_map_for_pos(
        commands, // FIXME: needs the position where the player is loaded.
        0, 0,
    );
}
