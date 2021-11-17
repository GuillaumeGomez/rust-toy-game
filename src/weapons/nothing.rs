use crate::character::Direction;
use crate::texture_holder::TextureId;
use crate::weapon::{Weapon, WeaponAction, WeaponActionKind, WeaponKind};
use crate::{GetDimension, ONE_SECOND};

#[derive(Debug)]
pub struct Nothing;

pub const RANGE: f32 = 15.;

impl Nothing {
    pub fn new(attack: i32) -> Weapon {
        Weapon {
            x: 0.,
            y: 0.,
            data_id: "",
            total_time: ONE_SECOND / 5,
            kind: WeaponKind::Nothing(Nothing),
            attack,
        }
    }
    pub fn use_it(&mut self, direction: Direction, total_duration: u32) -> Option<WeaponAction> {
        let (target_x, target_y) = match direction {
            Direction::Up => (0., -RANGE),
            Direction::Down => (0., RANGE),
            Direction::Left => (-RANGE, 0.),
            Direction::Right => (RANGE, 0.),
        };
        Some(WeaponAction {
            duration: 0,
            total_duration,
            x_add: 0.,
            y_add: 0.,
            kind: WeaponActionKind::AttackByMove { target_x, target_y },
        })
    }
    pub fn weight(&self) -> u32 {
        1
    }
    pub fn get_texture(&self) -> Option<TextureId> {
        None
    }
}

impl GetDimension for Nothing {
    fn width(&self) -> u32 {
        RANGE as _
    }
    fn height(&self) -> u32 {
        RANGE as _
    }
}
