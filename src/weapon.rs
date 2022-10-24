use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::character::Character;

#[derive(Debug, Component)]
pub struct Weapon {
    pub weight: f32,
}

pub fn check_receivers(
    characters: &Query<(Entity, &Character, &Children)>,
    weapon: &Weapon,
    weapon_id: &Entity,
    receiver: &Entity,
) {
    let (attacker_id, attacker) = match characters
        .iter()
        .find(|(e, c, children)| c.is_attacking && children.contains(weapon_id))
    {
        Some((e, a, _)) => (e, a),
        None => return,
    };
    let (receiver_id, receiver) = match characters
        .iter()
        .find(|(e, c, children)| children.contains(receiver))
    {
        Some((e, r, _)) => (e, r),
        None => return,
    };
    if attacker_id == receiver_id {
        println!("attacked itself :'(");
    } else {
        println!("Found both attacker and receiver!");
    }
}

pub fn handle_attacks(
    world: &World,
    mut collision_events: EventReader<CollisionEvent>,
    mut characters: Query<(Entity, &Character, &Children)>,
) {
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(x, y, CollisionEventFlags::SENSOR) = collision_event {
            match world.get_entity(*x).and_then(|e| e.get::<Weapon>()) {
                Some(w) => {
                    check_receivers(&characters, w, x, y);
                    continue;
                }
                None => {}
            };
            match world.get_entity(*y).and_then(|e| e.get::<Weapon>()) {
                Some(w) => check_receivers(&characters, w, y, x),
                None => {}
            };
        }
    }
}
