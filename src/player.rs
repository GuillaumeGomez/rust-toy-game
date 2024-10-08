use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{
    Character, CharacterAnimationInfo, CharacterAnimationType, CharacterKind, CharacterPoints,
    GrassEffectBundle,
};
use crate::inventory::Inventory;
use crate::weapon::Weapon;
use crate::RUN_STAMINA_CONSUMPTION_PER_SEC;

#[derive(Component)]
pub struct IsPlayer;

const PLAYER_WIDTH: u32 = 22;
const PLAYER_HEIGHT: u32 = 24;
const PLAYER_WIDTH_F: f32 = PLAYER_WIDTH as f32;
const PLAYER_HEIGHT_F: f32 = PLAYER_HEIGHT as f32;

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
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    const NB_ANIMATIONS: usize = 10;
    const ANIMATION_TIME: f32 = 0.08;

    // spawn player
    let texture_handle = asset_server.load("textures/player.png");
    let texture_atlas = TextureAtlasLayout::from_grid(
        UVec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
        NB_ANIMATIONS as _,
        5,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let weapon_handle = asset_server.load("textures/weapon.png");
    let mut character = Character::new(
        1,
        0,
        CharacterPoints::level_1(),
        PLAYER_WIDTH_F,
        PLAYER_HEIGHT_F,
        CharacterKind::Player,
    );

    const WEAPON_WIDTH: f32 = 7.;
    const WEAPON_HEIGHT: f32 = 20.;

    let weapon = Weapon::new(1, 1., WEAPON_WIDTH, WEAPON_HEIGHT);
    character.set_weapon(&weapon);

    commands
        .spawn((
            Player {
                is_running: false,
                waiting_for_rerun: false,
                old_x: 0.,
                old_y: 0.,
            },
            crate::character::CharacterBundle::new(
                character,
                CharacterAnimationInfo::new(
                    ANIMATION_TIME,
                    NB_ANIMATIONS,
                    CharacterAnimationType::ForwardIdle,
                ),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: PLAYER_WIDTH_F,
                            y: PLAYER_HEIGHT_F,
                        }),
                        ..default()
                    },
                    texture: texture_handle,
                    transform: Transform::from_xyz(0.0, 210.0, crate::CHARACTER_Z_INDEX),
                    ..default()
                },
                TextureAtlas {
                    index: 0,
                    layout: texture_atlas_handle,
                    ..default()
                },
                Inventory::new(40, 13, Some(weapon.clone())),
            ),
            RigidBody::Dynamic,
            Velocity::zero(),
            LockedAxes::ROTATION_LOCKED,
            Damping {
                linear_damping: 8.,
                angular_damping: 8.,
            },
        ))
        .with_children(|children| {
            // The "move" box.
            app_state.player_id = Some(
                children
                    .spawn((
                        Collider::cuboid(8.0, 7.0),
                        TransformBundle::from(Transform::from_xyz(0.0, -5.0, 0.0)),
                        ActiveEvents::COLLISION_EVENTS,
                        CollisionGroups::new(crate::OUTSIDE_WORLD, crate::OUTSIDE_WORLD),
                    ))
                    .id(),
            );
            // The hitbox.
            children.spawn((
                Collider::cuboid(PLAYER_WIDTH_F / 2. - 2., PLAYER_HEIGHT_F / 2. - 2.),
                TransformBundle::from(Transform::from_xyz(0.0, 2.0, 0.0)),
                Sensor,
                CollisionGroups::new(crate::HITBOX, crate::HITBOX),
            ));
            // The "interaction" hitbox.
            children.spawn((
                crate::character::Interaction,
                RigidBody::Dynamic,
                Collider::cuboid(1., 1.),
                TransformBundle::from(Transform::from_xyz(0.0, PLAYER_HEIGHT_F / -2. - 7., 0.)),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(crate::INTERACTION, crate::INTERACTION),
            ));
            // The weapon (invisible for the moment).
            children.spawn((
                weapon,
                IsPlayer,
                RigidBody::Dynamic,
                SpriteBundle {
                    texture: weapon_handle,
                    sprite: Sprite {
                        custom_size: Some(Vec2 {
                            x: WEAPON_WIDTH,
                            y: WEAPON_HEIGHT,
                        }),
                        ..default()
                    },
                    // We put the collision handler "outside" of the player to avoid triggering
                    // unwanted collision events.
                    transform: Transform::from_xyz(
                        PLAYER_WIDTH_F,
                        0.0,
                        crate::FURNITURE_TOP_PART_Z_INDEX - crate::CHARACTER_Z_INDEX + 0.2,
                    ),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                Collider::polyline(
                    vec![
                        Vec2 {
                            x: WEAPON_WIDTH / 4.,
                            y: WEAPON_HEIGHT / -6.,
                        },
                        Vec2 {
                            x: WEAPON_WIDTH / 4.,
                            y: WEAPON_HEIGHT / 2.,
                        },
                        Vec2 {
                            x: WEAPON_WIDTH / -4.,
                            y: WEAPON_HEIGHT / 2.,
                        },
                        Vec2 {
                            x: WEAPON_WIDTH / -4.,
                            y: WEAPON_HEIGHT / -6.,
                        },
                    ],
                    None,
                ),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(crate::NOTHING, crate::NOTHING),
            ));
            // The "grass effect" (invisible for the moment).
            children.spawn(GrassEffectBundle::new(PLAYER_HEIGHT_F, asset_server));
        });

    println!("player id: {:?}", app_state.player_id);
}

pub fn player_movement_system(
    timer: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_info: Query<(
        Entity,
        &mut Player,
        &mut TextureAtlas,
        &mut Velocity,
        &mut Character,
        &mut CharacterAnimationInfo,
    )>,
    mut player_interaction: Query<(&Parent, &mut Transform), With<crate::character::Interaction>>,
) {
    let (entity, mut player, mut sprite, mut rb_vels, mut character, mut animation) =
        match player_info.get_single_mut() {
            Ok(x) => x,
            _ => return,
        };
    let was_running = player.is_running;
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        if !player.waiting_for_rerun {
            let required_to_run = timer.delta().as_secs_f32() * RUN_STAMINA_CONSUMPTION_PER_SEC;
            player.is_running = character.stats.stamina.value() >= required_to_run;
            if was_running && !player.is_running {
                player.waiting_for_rerun = true;
            }
        } else if player.is_running {
            player.is_running = false;
        }
    } else if player.waiting_for_rerun || player.is_running {
        player.waiting_for_rerun = false;
        player.is_running = false;
    }

    let mut speed = character.stats.move_speed;
    if player.is_running {
        speed *= 2.;
    }

    let up = keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp);
    let down = keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown);
    let left = keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft);
    let right =
        keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight);

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
        if !is_equal {
            for (parent, mut transform) in player_interaction.iter_mut() {
                if parent.get() != entity {
                    continue;
                }
                match animation.animation_type {
                    CharacterAnimationType::ForwardMove | CharacterAnimationType::ForwardIdle => {
                        transform.translation.x = 0.;
                        transform.translation.y = PLAYER_HEIGHT_F / -2. - 7.;
                    }
                    CharacterAnimationType::BackwardMove | CharacterAnimationType::BackwardIdle => {
                        transform.translation.x = 0.;
                        transform.translation.y = PLAYER_HEIGHT_F / 2. + 7.;
                    }
                    CharacterAnimationType::LeftMove | CharacterAnimationType::LeftIdle => {
                        transform.translation.x = PLAYER_WIDTH_F / -2. - 4.;
                        transform.translation.y = 0.;
                    }
                    CharacterAnimationType::RightMove | CharacterAnimationType::RightIdle => {
                        transform.translation.x = PLAYER_WIDTH_F / 2. + 4.;
                        transform.translation.y = 0.;
                    }
                }
                break;
            }
        }
    }

    if player.is_running {
        // When runnning, 10 stamina a secs are computed.
        let before = character.stats.stamina.value();
        if !character
            .stats
            .stamina
            .subtract(timer.delta().as_secs_f32() * RUN_STAMINA_CONSUMPTION_PER_SEC)
        {
            player.waiting_for_rerun = true;
        }
    } else if !character.is_attacking && !character.stats.stamina.is_full() {
        // If the character regained enough stamina to run again for at least 3 seconds, we
        // switch it back automatically to running.
        if player.waiting_for_rerun
            && character.stats.stamina.value() > RUN_STAMINA_CONSUMPTION_PER_SEC * 3.
        {
            player.waiting_for_rerun = false;
        }
    }

    if skip_animation_update {
        return;
    }

    sprite.index = animation.animation_type.get_index(animation.nb_animations);
    if player.is_running {
        animation.timer = Timer::from_seconds(animation.animation_time / 2., TimerMode::Repeating);
    } else {
        animation.timer = Timer::from_seconds(animation.animation_time, TimerMode::Repeating);
    }
}

pub fn player_attack_system(
    timer: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Character, &Inventory, &CharacterAnimationInfo), With<Player>>,
    mut weapon_info: Query<
        (
            &mut Weapon,
            &mut Visibility,
            &mut Transform,
            &mut CollisionGroups,
        ),
        With<IsPlayer>,
    >,
) {
    let (ref mut character, inventory, animation_info) = player.single_mut();
    if inventory.equipped_weapon.is_none() {
        return;
    }
    let (mut weapon, mut visibility, mut transform, mut collision_groups) =
        match weapon_info.get_single_mut() {
            Ok(p) => p,
            Err(_) => return,
        };

    if character.is_attacking {
        let delta = timer.delta().as_secs_f32();
        character.attack_timer.tick(timer.delta());
        if character.attack_timer.finished()
            || !character
                .stats
                .stamina
                .subtract(delta * weapon.weight * 10.)
        {
            character.is_attacking = false;
        }
    } else if keyboard_input.pressed(KeyCode::Space) {
        character.is_attacking = character.stats.stamina.value()
            > weapon.weight * 10. * character.attack_timer.duration().as_secs_f32();
        if character.is_attacking {
            character.attack_timer.reset();
            *visibility = Visibility::Inherited;
            collision_groups.memberships = crate::HITBOX;
            collision_groups.filters = crate::HITBOX;
        }
    }
    if !character.is_attacking {
        *visibility = Visibility::Hidden;
        collision_groups.memberships = crate::NOTHING;
        collision_groups.filters = crate::NOTHING;
        return;
    }
    let percent =
        character.attack_timer.elapsed_secs() / character.attack_timer.duration().as_secs_f32();
    let angle = std::f32::consts::PI / 2. * percent - std::f32::consts::PI / 4.;
    transform.rotation = match animation_info.animation_type {
        CharacterAnimationType::ForwardIdle | CharacterAnimationType::ForwardMove => {
            transform.translation.y = PLAYER_HEIGHT_F / -2. - 8.;
            transform.translation.x = 5. * percent - 2.;
            Quat::from_rotation_z(std::f32::consts::PI + angle)
        }
        CharacterAnimationType::BackwardIdle | CharacterAnimationType::BackwardMove => {
            transform.translation.y = PLAYER_HEIGHT_F / 2. + 8.;
            transform.translation.x = -5. * percent + 3.;
            Quat::from_rotation_z(0. + angle)
        }
        CharacterAnimationType::LeftIdle | CharacterAnimationType::LeftMove => {
            transform.translation.y = -5. * percent + 1.;
            transform.translation.x = PLAYER_WIDTH_F / -2. - 5.;
            Quat::from_rotation_z(std::f32::consts::PI / 2. + angle)
        }
        CharacterAnimationType::RightIdle | CharacterAnimationType::RightMove => {
            transform.translation.y = 5. * percent;
            transform.translation.x = PLAYER_WIDTH_F / 2. + 5.;
            Quat::from_rotation_z(std::f32::consts::PI / -2. + angle)
        }
    };
}
