use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::{Character, CharacterPoints};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CharacterAnimationType {
    ForwardIdle,
    BackwardIdle,
    LeftIdle,
    RightIdle,
    ForwardMove,
    BackwardMove,
    LeftMove,
    RightMove,
}

impl CharacterAnimationType {
    fn is_idle(self) -> bool {
        matches!(
            self,
            CharacterAnimationType::ForwardIdle
                | CharacterAnimationType::BackwardIdle
                | CharacterAnimationType::LeftIdle
                | CharacterAnimationType::RightIdle
        )
    }

    fn get_index(self, nb_animations: usize) -> usize {
        match self {
            Self::ForwardIdle => 0,
            Self::BackwardIdle => 1,
            Self::LeftIdle => 2,
            Self::RightIdle => 3,
            Self::ForwardMove => nb_animations,
            Self::BackwardMove => nb_animations * 2,
            Self::LeftMove => nb_animations * 3,
            Self::RightMove => nb_animations * 4,
        }
    }

    fn stop_movement(&mut self) {
        match *self {
            Self::ForwardMove => *self = Self::ForwardIdle,
            Self::BackwardMove => *self = Self::BackwardIdle,
            Self::LeftMove => *self = Self::LeftIdle,
            Self::RightMove => *self = Self::RightIdle,
            _ => {}
        }
    }

    fn is_equal(self, x_axis: i8, y_axis: i8) -> bool {
        match self {
            Self::ForwardMove | Self::ForwardIdle => y_axis < 0,
            Self::BackwardMove | Self::BackwardIdle => y_axis > 0,
            Self::LeftMove | Self::LeftIdle => x_axis < 0,
            Self::RightMove | Self::RightIdle => x_axis > 0,
        }
    }

    fn set_move(&mut self, x_axis: i8, y_axis: i8) {
        if x_axis < 0 {
            *self = Self::LeftMove;
        } else if x_axis > 0 {
            *self = Self::RightMove;
        } else if y_axis < 0 {
            *self = Self::ForwardMove;
        } else {
            *self = Self::BackwardMove;
        }
    }
}

pub fn animate_character_system(
    time: Res<Time>,
    mut animation_query: Query<(&mut PlayerComponent, &mut TextureAtlasSprite)>,
) {
    for (mut player, mut sprite) in animation_query.iter_mut() {
        if !player.animation_type.is_idle() {
            player.timer.tick(time.delta());

            if player.timer.finished() {
                let max = player
                    .animation_type
                    .get_index(PlayerComponent::NB_ANIMATIONS)
                    + 9;
                if sprite.index == max {
                    sprite.index -= 9;
                } else {
                    sprite.index += 1;
                }
            }
        }
    }
}

#[derive(Debug, Component)]
pub struct PlayerComponent {
    pub speed: f32,
    pub timer: Timer,
    pub animation_type: CharacterAnimationType,
    pub character: Character,
    pub is_running: bool,
}

impl PlayerComponent {
    const ANIMATION_TIME: f32 = 0.08;
    const NB_ANIMATIONS: usize = 10;
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // spawn player
    let texture_handle = asset_server.load("textures/player.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(22., 24.),
        PlayerComponent::NB_ANIMATIONS,
        5,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn()
        .insert(PlayerComponent {
            speed: 100.0,
            timer: Timer::from_seconds(PlayerComponent::ANIMATION_TIME, true),
            animation_type: CharacterAnimationType::ForwardIdle,
            character: Character::new(1, 0, CharacterPoints::level_1()),
            is_running: false,
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::cuboid(2.0, 2.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 210.0, 0.0)));
}

pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_info: Query<(&mut PlayerComponent, &mut TextureAtlasSprite, &mut Velocity)>,
    // app_state: Res<State<GameState>>,
) {
    // if we are not playing the game prevent the player from moving
    // if app_state.current() != &GameState::MainGame {
    //     return;
    // }

    for (mut player, mut sprite, mut rb_vels) in player_info.iter_mut() {
        player.is_running = keyboard_input.pressed(KeyCode::LShift);
        let mut speed = player.speed;
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

        if x_axis == 0 && y_axis == 0 {
            player.animation_type.stop_movement();
            rb_vels.linvel.x = 0.;
            rb_vels.linvel.y = 0.;
        } else {
            let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
            move_delta /= move_delta.length();
            rb_vels.linvel = move_delta * speed;

            if player.animation_type.is_idle() || !player.animation_type.is_equal(x_axis, y_axis)
            {
                player.animation_type.set_move(x_axis, y_axis);
            } else {
                // Nothing to be updated.
                continue;
            }
        }

        sprite.index = player
            .animation_type
            .get_index(PlayerComponent::NB_ANIMATIONS);
        player.timer = Timer::from_seconds(PlayerComponent::ANIMATION_TIME, true);
        break;
    }
}
