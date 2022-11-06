use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterPoints,
};
use crate::weapon::Weapon;
use crate::RUN_STAMINA_CONSUMPTION_PER_SEC;

#[derive(Component)]
pub struct IsPlayer;

const PLAYER_WIDTH: f32 = 22.;
const PLAYER_HEIGHT: f32 = 24.;

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
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
        NB_ANIMATIONS,
        5,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let weapon_handle = asset_server.load("textures/weapon.png");
    let character = Character::new(
        1,
        0,
        CharacterPoints::level_1(),
        PLAYER_WIDTH,
        PLAYER_HEIGHT,
    );

    const WEAPON_WIDTH: f32 = 7.;
    const WEAPON_HEIGHT: f32 = 20.;

    let weapon = character.set_weapon(1, 1., WEAPON_WIDTH, WEAPON_HEIGHT);

    commands
        .spawn()
        .insert(Player {
            is_running: false,
            waiting_for_rerun: false,
            old_x: 0.,
            old_y: 0.,
        })
        .insert(character)
        .insert(CharacterAnimationInfo {
            animation_time: ANIMATION_TIME,
            nb_animations: NB_ANIMATIONS,
            timer: Timer::from_seconds(ANIMATION_TIME, true),
            animation_type: CharacterAnimationType::ForwardIdle,
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2 {
                    x: PLAYER_WIDTH,
                    y: PLAYER_HEIGHT,
                }),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(LockedAxes::ROTATION_LOCKED)
        .with_children(|children| {
            // The "move" box.
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
            // The hitbox.
            children
                .spawn()
                .insert(Collider::cuboid(
                    PLAYER_WIDTH / 2. - 2.,
                    PLAYER_HEIGHT / 2. - 2.,
                ))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 2.0, 0.0)))
                .insert(Sensor)
                .insert(CollisionGroups::new(crate::HITBOX, crate::HITBOX));
            // The weapon (invisible for the moment).
            children
                .spawn()
                .insert(weapon)
                .insert(IsPlayer)
                .insert(RigidBody::Dynamic)
                .insert_bundle(SpriteBundle {
                    texture: weapon_handle,
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: WEAPON_WIDTH,
                            y: WEAPON_HEIGHT,
                        }),
                        ..default()
                    },
                    ..default()
                })
                // We put the collision handler "outside" of the player to avoid triggering unwanted
                // collision events.
                .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    PLAYER_WIDTH,
                    0.0,
                    0.0,
                )))
                .insert(Collider::cuboid(WEAPON_WIDTH / 2. - 1., WEAPON_HEIGHT / 2.))
                .insert(Visibility { is_visible: false })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(CollisionGroups::new(crate::HITBOX, crate::HITBOX));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 210.0, 1.0)));

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

            let is_equal = animation.animation_type.is_equal(x_axis, y_axis);
            if animation.animation_type.is_idle() || !is_equal {
                if !is_equal {
                    // If the character changes direction, it stops the attack.
                    character.is_attacking = false;
                }
                animation.animation_type.set_move(x_axis, y_axis);
            } else if player.is_running == was_running {
                // Nothing to be updated.
                skip_animation_update = true;
            }
        }

        if player.is_running {
            // When runnning, 10 stamina a secs are computed.
            if !character
                .stats
                .stamina
                .subtract(timer.delta().as_secs_f32() * RUN_STAMINA_CONSUMPTION_PER_SEC)
            {
                player.waiting_for_rerun = true;
            }
        } else if !character.is_attacking && !character.stats.stamina.is_full() {
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

pub fn player_attack_system(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<(&mut Character, &CharacterAnimationInfo), With<Player>>,
    mut weapon_info: Query<(&mut Weapon, &mut Visibility, &mut Transform), With<IsPlayer>>,
) {
    let (mut weapon, mut visibility, mut transform) = match weapon_info.get_single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };
    let (ref mut character, animation_info) = player.single_mut();

    if character.is_attacking {
        let delta = timer.delta().as_secs_f32();
        weapon.timer.tick(timer.delta());
        if weapon.timer.finished()
            || !character
                .stats
                .stamina
                .subtract(delta * weapon.weight * 10.)
        {
            character.is_attacking = false;
            visibility.is_visible = false;
        }
    } else if visibility.is_visible {
        visibility.is_visible = false;
    } else if keyboard_input.pressed(KeyCode::Space) {
        character.is_attacking = character.stats.stamina.value() > weapon.weight * 9.;
        weapon.timer.reset();
        visibility.is_visible = true;
        // We set its z-index to 1 so it also appears in buildings.
        transform.translation.z = 0.9;
    }
    if !character.is_attacking {
        return;
    }
    let percent = weapon.timer.elapsed_secs() / weapon.timer.duration().as_secs_f32();
    let angle = std::f32::consts::PI / 2. * percent - std::f32::consts::PI / 4.;
    transform.rotation = match animation_info.animation_type {
        CharacterAnimationType::ForwardIdle | CharacterAnimationType::ForwardMove => {
            transform.translation.y = PLAYER_HEIGHT / -2. - 8.;
            transform.translation.x = 5. * percent - 2.;
            Quat::from_rotation_z(std::f32::consts::PI + angle)
        }
        CharacterAnimationType::BackwardIdle | CharacterAnimationType::BackwardMove => {
            transform.translation.y = PLAYER_HEIGHT / 2. + 8.;
            transform.translation.x = -5. * percent + 3.;
            Quat::from_rotation_z(0. + angle)
        }
        CharacterAnimationType::LeftIdle | CharacterAnimationType::LeftMove => {
            transform.translation.y = -5. * percent + 1.;
            transform.translation.x = PLAYER_WIDTH / -2. - 5.;
            Quat::from_rotation_z(std::f32::consts::PI / 2. + angle)
        }
        CharacterAnimationType::RightIdle | CharacterAnimationType::RightMove => {
            transform.translation.y = 5. * percent;
            transform.translation.x = PLAYER_WIDTH / 2. + 5.;
            Quat::from_rotation_z(std::f32::consts::PI / -2. + angle)
        }
    };
}
