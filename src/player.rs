use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterPoints,
};
use crate::RUN_STAMINA_CONSUMPTION_PER_SEC;

#[derive(Debug, Component)]
pub struct Player {
    pub is_running: bool,
    // Once stamina is completely consumed, we need to wait for SHIFT to be released before
    // running again.
    pub waiting_for_rerun: bool,
    // Used when switching between outside/inside.
    pub old_x: f32,
    pub old_y: f32,
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<crate::GameInfo>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    const NB_ANIMATIONS: usize = 10;
    const ANIMATION_TIME: f32 = 0.08;

    // spawn player
    let texture_handle = asset_server.load("textures/player.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(22., 24.), NB_ANIMATIONS, 5);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn()
        .insert(Player {
            is_running: false,
            waiting_for_rerun: false,
            old_x: 0.,
            old_y: 0.,
        })
        .insert(Character::new(1, 0, CharacterPoints::level_1()))
        .insert(CharacterAnimationInfo {
            animation_time: ANIMATION_TIME,
            nb_animations: NB_ANIMATIONS,
            timer: Timer::from_seconds(ANIMATION_TIME, true),
            animation_type: CharacterAnimationType::ForwardIdle,
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(LockedAxes::ROTATION_LOCKED)
        .with_children(|children| {
            app_state.player_id = Some(
                children
                    .spawn()
                    .insert(Collider::cuboid(8.0, 7.0))
                    .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -5.0, 0.0)))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(CollisionGroups::new(
                        crate::OUTSIDE_WORLD,
                        crate::OUTSIDE_WORLD,
                    ))
                    .id(),
            );
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 210.0, 0.0)));

    println!("player id: {:?}", app_state.player_id);
}

pub fn player_movement_system(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_info: Query<(
        &mut Player,
        &mut TextureAtlasSprite,
        &mut Velocity,
        &mut Character,
        &mut CharacterAnimationInfo,
    )>,
) {
    for (mut player, mut sprite, mut rb_vels, mut character, mut animation) in
        player_info.iter_mut()
    {
        let was_running = player.is_running;
        if keyboard_input.pressed(KeyCode::LShift) {
            if !player.waiting_for_rerun {
                let required_to_run = timer.delta().as_secs_f32() * RUN_STAMINA_CONSUMPTION_PER_SEC;
                player.is_running = character.stats.stamina.value() >= required_to_run;
                if was_running && !player.is_running {
                    player.waiting_for_rerun = true;
                }
            } else if player.is_running {
                player.is_running = false;
            }
        } else if player.waiting_for_rerun {
            player.waiting_for_rerun = false;
        }

        let mut speed = character.stats.move_speed;
        if player.is_running {
            speed *= 2.;
        }

        let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
        let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
        let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

        // convert to axis multipliers
        let x_axis = -(left as i8) + right as i8;
        let y_axis = -(down as i8) + up as i8;

        let mut skip_animation_update = false;
        if x_axis == 0 && y_axis == 0 {
            animation.animation_type.stop_movement();
            rb_vels.linvel.x = 0.;
            rb_vels.linvel.y = 0.;

            // If we don't move, nothing to update.
            player.is_running = false;
            player.waiting_for_rerun = false;
        } else {
            let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
            move_delta /= move_delta.length();
            rb_vels.linvel = move_delta * speed;

            if animation.animation_type.is_idle()
                || !animation.animation_type.is_equal(x_axis, y_axis)
            {
                animation.animation_type.set_move(x_axis, y_axis);
            } else if player.is_running == was_running {
                // Nothing to be updated.
                skip_animation_update = true;
            }
        }

        if player.is_running {
            // When runnning, 10 stamina a secs are computed.
            character
                .stats
                .stamina
                .subtract(timer.delta().as_secs_f32() * RUN_STAMINA_CONSUMPTION_PER_SEC);
            if character.stats.stamina.is_empty() {
                player.waiting_for_rerun = true;
            }
        } else if !character.stats.stamina.is_full() {
            character.stats.stamina.refresh(timer.delta().as_secs_f32());
            // If the character regained enough stamina to run again for at least 3 seconds, we
            // switch it back automatically to running.
            if player.waiting_for_rerun
                && character.stats.stamina.value() > RUN_STAMINA_CONSUMPTION_PER_SEC * 3.
            {
                player.waiting_for_rerun = false;
            }
        }

        if skip_animation_update {
            continue;
        }

        sprite.index = animation.animation_type.get_index(animation.nb_animations);
        if player.is_running {
            animation.timer = Timer::from_seconds(animation.animation_time / 2., true);
        } else {
            animation.timer = Timer::from_seconds(animation.animation_time, true);
        }
        break;
    }
}
