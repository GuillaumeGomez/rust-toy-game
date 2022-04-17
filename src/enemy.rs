use crate::character::{Character, Direction};
use crate::map::Map;
use crate::player::Player;
use crate::{GetDimension, GetPos, Id};

// TODO: add a "LookAround" state where the NPC just look around.
#[derive(Clone, Debug)]
pub enum EnemyAction {
    // Not doing anything for the moment...
    None,
    Attack(Direction),
    MoveTo(Vec<(f32, f32)>),
    // Targetted player.
    MoveToPlayer(Id, Vec<(f32, f32)>),
}

impl EnemyAction {
    #[allow(dead_code)]
    pub fn is_move_to_player(&self) -> bool {
        matches!(*self, Self::MoveToPlayer(_, _))
    }

    pub fn is_attack(&self) -> bool {
        matches!(*self, Self::Attack(_))
    }
}

pub trait Enemy: GetPos + GetDimension {
    fn character(&self) -> &Character;
    fn character_mut(&mut self) -> &mut Character;
    fn apply_move(
        &self,
        map: &Map,
        elapsed: u32,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
    ) -> (f32, f32);
    fn draw(&mut self, system: &mut crate::system::System, debug: bool);
    fn update(&mut self, elapsed: u32, x: f32, y: f32);
    fn id(&self) -> Id;
}

impl<E: Enemy> GetPos for E {
    fn x(&self) -> f32 {
        self.character().x()
    }

    fn y(&self) -> f32 {
        self.character().y()
    }
}

impl<E: Enemy> GetDimension for E {
    fn width(&self) -> u32 {
        self.character().width()
    }
    fn height(&self) -> u32 {
        self.character().height()
    }
}
