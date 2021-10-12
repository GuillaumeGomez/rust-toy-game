use crate::character::{Character, Direction};
use crate::enemies::Skeleton;
use crate::map::Map;
use crate::player::Player;
use crate::{GetDimension, GetPos, Id};

// TODO: add a "LookAround" state where the NPC just look around.
#[derive(Clone, Debug)]
pub enum EnemyAction {
    // Not doing anything for the moment...
    None,
    Attack(Direction),
    MoveTo(Vec<(i64, i64)>),
    // Targetted player (in case of multiplayer, might be nice to have IDs for players)
    MoveToPlayer(Vec<(i64, i64)>),
}

impl EnemyAction {
    #[allow(dead_code)]
    pub fn is_move_to_player(&self) -> bool {
        match *self {
            Self::MoveToPlayer(_) => true,
            _ => false,
        }
    }

    pub fn is_attack(&self) -> bool {
        match *self {
            Self::Attack(_) => true,
            _ => false,
        }
    }
}

pub trait Enemy: GetPos + GetDimension {
    fn character(&self) -> &Character;
    fn character_mut(&mut self) -> &mut Character;
    fn apply_move(
        &self,
        map: &Map,
        elapsed: u64,
        players: &[Player],
        npcs: &[Box<Skeleton>],
    ) -> (i64, i64);
    fn draw(&mut self, system: &mut crate::system::System, debug: bool);
    fn update(&mut self, elapsed: u64, x: i64, y: i64);
    fn id(&self) -> Id;
}
