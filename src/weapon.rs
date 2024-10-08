use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

use crate::character::{Character, CharacterKind};
use crate::environment::Grass;

const NOTIFICATION_MOVE: f32 = 5.;
const NOTIFICATION_TIME: f32 = 0.5;

#[derive(Debug, Component)]
pub struct Notification {
    pub timer: Timer,
}

#[derive(Debug, Component, Clone)]
pub struct Weapon {
    pub attack: u32,
    pub weight: f32,
    pub width: f32,
    pub height: f32,
}

impl Weapon {
    pub fn new(attack: u32, weight: f32, width: f32, height: f32) -> Self {
        Self {
            attack,
            weight,
            width,
            height,
        }
    }
}

pub fn update_notifications(
    mut commands: Commands,
    timer: Res<Time>,
    mut notifications: Query<(Entity, &mut Notification, &mut Transform)>,
) {
    let delta = timer.delta();
    let delta_secs = delta.as_secs_f32();
    for (entity, mut notif, mut pos) in notifications.iter_mut() {
        notif.timer.tick(delta);
        if notif.timer.finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            pos.translation.y += NOTIFICATION_MOVE * delta_secs / NOTIFICATION_TIME;
        }
    }
}

pub fn check_receivers(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    characters: &mut Query<(Entity, &mut Character, &Children)>,
    attack: u32,
    attacker_id: Entity,
    attacker_kind: CharacterKind,
    receiver: &Entity,
) {
    let (receiver_id, mut receiver) = match characters
        .iter_mut()
        .find(|(_, _, children)| children.contains(receiver))
    {
        Some((e, r, _)) => (e, r),
        None => return,
    };
    eprintln!("found receiver {:?} {:?}", attacker_id, receiver_id);
    // If attacker_id == receiver_id, it means the character attacked itself so we ignore it.
    // Also, we don't want monsters to attack their own.
    if attacker_id != receiver_id && attacker_kind != receiver.kind {
        let mut damage = attack.saturating_sub(receiver.stats.defense);
        if damage < 1 {
            damage = 1;
        }
        receiver.stats.health.subtract(damage as _);
        if receiver.stats.health.is_empty() {
            // TODO: add xp to the killer
            commands.entity(receiver_id).despawn_recursive();
        } else {
            let child = commands
                .spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            damage.to_string().as_str(),
                            TextStyle {
                                font: asset_server.load(crate::FONT),
                                font_size: 11.0,
                                color: Color::LinearRgba(LinearRgba::RED),
                            },
                        )
                        .with_justify(JustifyText::Center),
                        transform: Transform::from_xyz(0., receiver.height / 2. + 8., 1.),
                        ..default()
                    },
                    Notification {
                        timer: Timer::from_seconds(NOTIFICATION_TIME, TimerMode::Once),
                    },
                ))
                .id();
            commands.entity(receiver_id).add_child(child);
        }
    }
}

#[derive(Component)]
pub struct EntityDestroyer(Entity, Timer);

pub fn update_entity_destroyer(
    timer: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut EntityDestroyer)>,
) {
    let delta = timer.delta();
    for (e, mut destroyer) in query.iter_mut() {
        destroyer.1.tick(delta);
        if destroyer.1.finished() {
            commands.entity(destroyer.0).despawn_recursive();
            commands.entity(e).despawn_recursive();
        }
    }
}

fn check_grass(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    grass: &mut Query<(Entity, &Grass, &mut Transform)>,
    receiver: &Entity,
) -> bool {
    if let Ok((_, _, mut transform)) = grass.get_mut(*receiver) {
        // We "remove" the existing grass...
        commands.spawn(EntityDestroyer(
            *receiver,
            Timer::from_seconds(0.1, TimerMode::Once),
        ));

        // And replace it with a cut one.
        let cut_grass = asset_server.load("textures/cut-grass.png");
        commands.spawn((
            crate::game::OutsideWorld,
            SpriteBundle {
                texture: cut_grass,
                transform: transform.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2 {
                        x: crate::GRASS_SIZE * 3. / 4.,
                        y: crate::GRASS_SIZE * 3. / 4.,
                    }),
                    ..default()
                },
                ..default()
            },
        ));
        // Moving grass outside of view.
        transform.translation.y -= crate::MAP_SIZE * 3.;
        true
    } else {
        false
    }
}

macro_rules! getter {
    ($characters:ident, $weapons:ident, $x:ident, $y:ident) => {
        $characters
            .iter()
            .find(|(_, c, children)| c.is_attacking && children.contains($x))
            .and_then(|(attacker_id, attacker, children)| {
                if let Some((_, weapon)) =
                    $weapons.iter().find(|(id, weapon)| children.contains(id))
                {
                    Some((
                        attacker.stats.attack + weapon.attack,
                        attacker_id,
                        $y,
                        attacker.kind,
                    ))
                } else {
                    None
                }
            })
    };
}

// This macro calls te same thing but inverts `x` and `y` to get attacker and receiver.
macro_rules! get_attacker_and_receiver {
    ($characters:ident, $weapons:ident, $x:ident, $y:ident) => {
        if let Some((attack, attacker_id, receiver, attacker_kind)) =
            getter!($characters, $weapons, $x, $y)
        {
            (attack, attacker_id, receiver, attacker_kind)
        } else if let Some((attack, attacker_id, receiver, attacker_kind)) =
            getter!($characters, $weapons, $y, $x)
        {
            (attack, attacker_id, receiver, attacker_kind)
        } else {
            continue;
        }
    };
}

pub fn handle_attacks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    mut characters: Query<(Entity, &mut Character, &Children)>,
    mut grass: Query<(Entity, &Grass, &mut Transform)>,
    weapons: Query<(Entity, &Weapon)>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) = collision_event {
            let (attack, attacker_id, receiver, attacker_kind): (
                u32,
                Entity,
                &Entity,
                CharacterKind,
            ) = get_attacker_and_receiver!(characters, weapons, x, y);
            eprintln!("Found attacker");
            if !check_grass(&mut commands, &asset_server, &mut grass, receiver) {
                // if the attack didn't cut grass, then it's very likely a `Character`.
                check_receivers(
                    &mut commands,
                    &asset_server,
                    &mut characters,
                    attack,
                    attacker_id,
                    attacker_kind,
                    receiver,
                );
            }
        }
    }
}
