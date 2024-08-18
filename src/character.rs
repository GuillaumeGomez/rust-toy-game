use crate::inventory::Inventory;
use crate::stat::Stat;
use crate::weapon::Weapon;
use crate::STAT_POINTS_PER_LEVEL;

use bevy::ecs::component::Component;
use bevy::prelude::*;
use bevy_rapier2d::prelude::CollisionEvent;
use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

#[derive(Component)]
pub struct GrassEffect {
    // If the count is 0, then we don't display the grass effect.
    pub count: isize,
}

pub struct GrassEffectBundle;

impl GrassEffectBundle {
    pub fn new(parent_height: f32, asset_server: Res<AssetServer>) -> (GrassEffect, SpriteBundle) {
        (
            GrassEffect { count: 0 },
            SpriteBundle {
                texture: asset_server.load("textures/grass-effect.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2 { x: 18., y: 7. }),
                    ..default()
                },
                transform: Transform::from_xyz(0., parent_height / -2. + 3., 1.0),
                visibility: Visibility::Hidden,
                ..default()
            },
        )
    }
}

#[derive(Bundle)]
pub struct CharacterBundle {
    character: Character,
    animation_info: CharacterAnimationInfo,
    sprite_info: SpriteSheetBundle,
    inventory: Inventory,
}

impl CharacterBundle {
    pub fn new(
        character: Character,
        animation_info: CharacterAnimationInfo,
        sprite_info: SpriteSheetBundle,
        inventory: Inventory,
    ) -> Self {
        Self {
            character,
            animation_info,
            sprite_info,
            inventory,
        }
    }
}

#[derive(Component)]
pub struct CharacterInfo;
#[derive(Component)]
pub struct CharacterHealthBar;
#[derive(Component)]
pub struct CharacterHealthBarInner;

#[derive(Debug)]
pub struct CharacterStats {
    pub health: Stat,
    pub mana: Stat,
    pub stamina: Stat,
    pub defense: u32,
    pub attack: u32,
    /// The higher it is, the less time it takes to perform an attack.
    pub attack_speed: f32,
    // FIXME: this isn't used at the moment.
    pub magical_attack: u32,
    pub magical_defense: u32,
    /// It also takes into account the opponent level, agility and dexterity.
    pub dodge_change: u32,
    /// It also takes into account the opponent level, agility and dexterity.
    pub critical_attack_chance: u32,
    /// How far you go in one second.
    pub move_speed: f32,
}

#[derive(Debug)]
pub struct CharacterPoints {
    pub strength: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub stamina: u32,
    pub agility: u32,
    pub dexterity: u32,
}

impl CharacterPoints {
    pub fn level_1() -> Self {
        Self {
            strength: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            stamina: 1,
            agility: 1,
            dexterity: 1,
        }
    }

    pub fn assigned_points(&self) -> u32 {
        // All fields should be listed here.
        self.strength
            + self.constitution
            + self.intelligence
            + self.wisdom
            + self.stamina
            + self.agility
            + self.dexterity
    }

    pub fn generate_stats(&self, level: u16) -> CharacterStats {
        // character points
        //
        // At each level, health and mana raise a bit
        //
        // strength -> raises attack (and health a bit, defense a bit)
        // constitution -> raises health and life regen (and attack a bit, and defense a bit)
        // intelligence -> raises mana (and magical attack/defense a bit, and mana regen a bit)
        // wisdom -> raise magical attack/defense (and mana a bit, and mana regen a bit)
        // stamina -> raises endurance (and health a bit, and defense a bit, and endurance regen a bit)
        // agility -> raises attack speed a bit x 2, dodge change a bit, critical hit a bit, move speed a bit
        // dexterity -> raises critical hit a bit x 2, attack a bit, attack speed a bit

        let level = level as u32;
        // We start with 100 HP (95 + 5 * 1).
        let total_health =
            95 + 5 * level + 10 * self.constitution + 2 * self.strength + 2 * self.stamina;
        let health_regen_speed = 1. + 1. * (self.constitution as f32);
        // We start with 20 MP (16 + 4 * 1).
        let total_mana = 16 + 4 * level + 10 * self.intelligence + 4 * self.wisdom;
        let mana_regen_speed = 0.5 + 0.5 * (self.intelligence as f32) + 0.5 * (self.wisdom as f32);
        // We start with 50 SP.
        let total_stamina = 50 + self.stamina;
        let stamina_regen_speed = 4. + self.stamina as f32 / 2.;
        CharacterStats {
            health: Stat::new(health_regen_speed, total_health as _),
            mana: Stat::new(mana_regen_speed, total_mana as _),
            stamina: Stat::new(stamina_regen_speed, total_stamina as _),
            defense: (2 * self.constitution + 1) * self.stamina,
            attack: level + 5 * self.strength + self.constitution / 2 + self.dexterity / 2,
            attack_speed: (1 + 2 * self.agility + self.dexterity) as f32,
            // FIXME: for now this is useless.
            magical_attack: level + 2 * self.wisdom + self.intelligence,
            magical_defense: level / 2 + self.wisdom + self.intelligence / 2,
            dodge_change: level + self.agility,
            critical_attack_chance: level + 2 * self.dexterity + self.agility,
            // You gain 1% of speed every eight level.
            move_speed: 100. + ((level - 1) as f32) / 800.,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharacterKind {
    Player,
    Human,
    Monster,
}

#[derive(Component, Debug)]
pub struct Character {
    // FIXME: Move it into its own?
    pub kind: CharacterKind,
    pub xp_to_next_level: u64,
    pub xp: u64,
    pub level: u16,
    pub stats: CharacterStats,
    pub points: CharacterPoints,
    pub unused_points: u32,
    pub is_attacking: bool,
    pub width: f32,
    pub height: f32,
}

fn compute_xp_to_next_level(level: u16) -> u64 {
    let mut x = 100;
    for _ in 0..level {
        x += x - x / 2;
    }
    x
}

fn compute_total_nb_points(level: u16) -> u32 {
    let mut nb_points = 0;
    for _ in 1..level {
        nb_points += STAT_POINTS_PER_LEVEL;
    }
    nb_points
}

impl Character {
    pub fn new(
        level: u16,
        xp: u64,
        points: CharacterPoints,
        width: f32,
        height: f32,
        kind: CharacterKind,
    ) -> Self {
        let stats = points.generate_stats(level);
        let unassigned = points.assigned_points();
        Self {
            xp_to_next_level: compute_xp_to_next_level(level),
            xp,
            level,
            stats,
            points,
            unused_points: unassigned + compute_total_nb_points(level),
            is_attacking: false,
            width,
            height,
            kind,
        }
    }

    pub fn increase_xp(&mut self, xp_to_add: u64) {
        self.xp += xp_to_add;
        if self.xp >= self.xp_to_next_level {
            self.level += 1;
            self.xp -= self.xp_to_next_level;
            self.xp_to_next_level = self.xp_to_next_level + self.xp_to_next_level / 2;
            self.reset_stats();
            self.stats = self.points.generate_stats(self.level);
            self.unused_points += STAT_POINTS_PER_LEVEL;
            // self.animations.push(Animation::new_level_up(textures));
        }
    }

    pub fn use_stat_point(&mut self) {
        // FIXME: save the new character status on disk?
        self.unused_points =
            self.level as u32 * STAT_POINTS_PER_LEVEL.saturating_sub(self.points.assigned_points());
        self.stats = self.points.generate_stats(self.level);
    }

    pub fn reset_stats(&mut self) {
        self.stats.health.reset();
        self.stats.mana.reset();
        self.stats.stamina.reset();
    }

    pub fn set_weapon(
        &self,
        attack: u32,
        weapon_weigth: f32,
        weapon_width: f32,
        weapon_height: f32,
    ) -> Weapon {
        let mut time_for_an_attack = 333. - self.stats.attack_speed / 10.;
        if time_for_an_attack < 50. {
            time_for_an_attack = 50.;
        }
        Weapon::new(
            attack,
            weapon_weigth,
            weapon_height,
            weapon_width,
            time_for_an_attack,
        )
    }
}

#[derive(Component)]
pub struct CharacterAnimationInfo {
    pub animation_time: f32,
    pub nb_animations: usize,
    pub timer: Timer,
    pub animation_type: CharacterAnimationType,
    pub start_index: usize,
    pub play_once: bool,
}

impl CharacterAnimationInfo {
    pub fn new(
        animation_time: f32,
        nb_animations: usize,
        animation_type: CharacterAnimationType,
    ) -> Self {
        Self::new_with_start_index(animation_time, nb_animations, animation_type, 0)
    }

    pub fn new_with_start_index(
        animation_time: f32,
        nb_animations: usize,
        animation_type: CharacterAnimationType,
        start_index: usize,
    ) -> Self {
        Self {
            animation_time,
            nb_animations,
            timer: Timer::from_seconds(animation_time, TimerMode::Repeating),
            animation_type,
            start_index,
            play_once: false,
        }
    }

    pub fn new_once(
        animation_time: f32,
        nb_animations: usize,
        animation_type: CharacterAnimationType,
    ) -> Self {
        Self::new_once_with_start_index(animation_time, nb_animations, animation_type, 0)
    }

    pub fn new_once_with_start_index(
        animation_time: f32,
        nb_animations: usize,
        animation_type: CharacterAnimationType,
        start_index: usize,
    ) -> Self {
        Self {
            animation_time,
            nb_animations,
            timer: Timer::from_seconds(animation_time, TimerMode::Repeating),
            animation_type,
            start_index,
            play_once: true,
        }
    }
}

pub fn animate_character_system(
    time: Res<Time>,
    mut animation_query: Query<(&mut CharacterAnimationInfo, &mut TextureAtlas)>,
) {
    for (mut animation, mut sprite) in animation_query.iter_mut() {
        if !animation.animation_type.is_idle() {
            animation.timer.tick(time.delta());

            if animation.timer.finished() {
                if animation.play_once
                    && sprite.index + 1 - animation.start_index >= animation.nb_animations
                {
                    animation.timer.pause();
                }

                sprite.index = (sprite.index + 1 - animation.start_index) % animation.nb_animations
                    + animation.animation_type.get_index(animation.nb_animations)
                    + animation.start_index;
            }
        }
    }
}

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
    pub fn is_idle(self) -> bool {
        matches!(
            self,
            CharacterAnimationType::ForwardIdle
                | CharacterAnimationType::BackwardIdle
                | CharacterAnimationType::LeftIdle
                | CharacterAnimationType::RightIdle
        )
    }

    pub fn get_index(self, nb_animations: usize) -> usize {
        match self {
            Self::ForwardMove => 0,
            Self::BackwardMove => nb_animations,
            Self::LeftMove => nb_animations * 2,
            Self::RightMove => nb_animations * 3,
            Self::ForwardIdle => nb_animations * 4,
            Self::BackwardIdle => nb_animations * 4 + 1,
            Self::LeftIdle => nb_animations * 4 + 2,
            Self::RightIdle => nb_animations * 4 + 3,
        }
    }

    pub fn stop_movement(&mut self) {
        match *self {
            Self::ForwardMove => *self = Self::ForwardIdle,
            Self::BackwardMove => *self = Self::BackwardIdle,
            Self::LeftMove => *self = Self::LeftIdle,
            Self::RightMove => *self = Self::RightIdle,
            _ => {}
        }
    }

    pub fn is_equal(self, x_axis: i8, y_axis: i8) -> bool {
        match self {
            Self::ForwardMove | Self::ForwardIdle => y_axis < 0,
            Self::BackwardMove | Self::BackwardIdle => y_axis > 0,
            Self::LeftMove | Self::LeftIdle => x_axis < 0,
            Self::RightMove | Self::RightIdle => x_axis > 0,
        }
    }

    pub fn set_move(&mut self, x_axis: i8, y_axis: i8) {
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

pub fn refresh_characters_stats(timer: Res<Time>, mut characters: Query<&mut Character>) {
    let delta = timer.delta().as_secs_f32();
    for mut character in characters.iter_mut() {
        // stamina doesn't regen when attacking.
        if !character.is_attacking {
            character.stats.stamina.refresh(delta);
        }
        character.stats.health.refresh(delta);
        character.stats.mana.refresh(delta);
    }
}

#[derive(Component)]
pub struct Interaction;
#[derive(Component)]
pub struct InteractionText;

pub fn interaction_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    characters: Query<(Entity, &Character, &Children), Without<crate::player::Player>>,
    player: Query<&Children, With<crate::player::Player>>,
    interactions: Query<Entity, With<Interaction>>,
    mut interaction_texts: Query<Entity, With<InteractionText>>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) => {
                if !interactions.contains(*x) {
                    // No need to check anything if it's not an `Interaction` event!
                    continue;
                }
                let player_children = match player.get_single() {
                    Ok(x) => x,
                    _ => return,
                };
                let character_entity = if player_children.contains(x) {
                    y
                } else if player_children.contains(y) {
                    x
                } else {
                    continue;
                };
                for (entity, character, children) in characters.iter() {
                    if children.contains(character_entity) {
                        let child = commands
                            .spawn((
                                InteractionText,
                                Text2dBundle {
                                    text: Text::from_section(
                                        "Press ENTER to talk",
                                        TextStyle {
                                            font: asset_server.load(crate::FONT),
                                            font_size: 9.0,
                                            color: Color::WHITE,
                                        },
                                    )
                                    .with_justify(JustifyText::Center),
                                    transform: Transform::from_xyz(0., character.height / 2., 1.),
                                    ..default()
                                },
                            ))
                            .id();
                        commands.entity(entity).add_child(child);
                        break;
                    }
                }
            }
            CollisionEvent::Stopped(x, y, CollisionEventFlags::SENSOR) => {
                if !interactions.contains(*x) {
                    // No need to check anything if it's not an `Interaction` event!
                    continue;
                }
                let player_children = match player.get_single() {
                    Ok(x) => x,
                    _ => return,
                };
                let character_entity = if player_children.contains(x) {
                    y
                } else if player_children.contains(y) {
                    x
                } else {
                    continue;
                };
                for (entity, character, children) in characters.iter() {
                    if children.contains(character_entity) {
                        for child in children.iter() {
                            if let Ok(interaction) = interaction_texts.get(*child) {
                                commands.entity(interaction).despawn_recursive();
                                break;
                            }
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
