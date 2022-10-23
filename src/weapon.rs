use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::Character;

#[derive(Debug, Component)]
pub struct Weapon {
    pub weight: f32,
}

pub fn check_receivers(
    characters: &Query<(&Character, &Children)>,
    weapon: &Weapon,
    receiver: &Entity,
) {
    for (mut character, children) in characters.iter() {
        if children.contains(receiver) {
            println!("Found both attacker and receiver!");
            break;
        }
    }
}

pub fn handle_attacks(
    world: &World,
    mut collision_events: EventReader<CollisionEvent>,
    mut characters: Query<(&Character, &Children)>,
) {
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) = collision_event {
            match world.get_entity(*x).and_then(|e| e.get::<Weapon>()) {
                Some(w) => {
                    check_receivers(&characters, w, y);
                    continue;
                }
                None => {}
            };
            match world.get_entity(*y).and_then(|e| e.get::<Weapon>()) {
                Some(w) => {
                    check_receivers(&characters, w, x);
                    continue;
                }
                None => {}
            };
        }
    }
}
