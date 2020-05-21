use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::ops::{Deref, DerefMut};

use rand::Rng;
use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::character::{Action, Character, CharacterKind, Direction};
use crate::death_animation::DeathAnimation;
use crate::map::Map;
use crate::player::Player;
use crate::stat::Stat;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::utils;
use crate::{
    GetDimension, GetPos, Id, MAP_CASE_SIZE, MAX_DISTANCE_PURSUIT, MAX_DISTANCE_WANDERING,
    ONE_SECOND,
};

#[derive(Eq, Debug)]
struct Node {
    x: i64,
    y: i64,
    cost: u32,
    heuristic: u32,
}
impl Node {
    fn new(x: i64, y: i64, cost: u32) -> Node {
        Node {
            x,
            y,
            cost,
            heuristic: 0,
        }
    }
    fn compute_heuristic(&mut self, destination: &(i64, i64)) {
        self.heuristic = utils::compute_distance(&(self.x, self.y), destination) as u32 + self.cost;
    }
}
impl PartialEq<Node> for Node {
    fn eq(&self, other: &Node) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl PartialEq<Reverse<Node>> for Node {
    fn eq(&self, other: &Reverse<Node>) -> bool {
        self.x == other.0.x && self.y == other.0.y
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.heuristic.partial_cmp(&other.heuristic)
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).expect("cmp failed")
    }
}

// TODO: add a "LookAround" state where the NPC just look around.
#[derive(Clone, Debug)]
enum EnemyAction {
    // Not doing anything for the moment...
    None,
    MoveTo(Vec<(i64, i64)>),
    // Targetted player (in case of multiplayer, might be nice to have IDs for players)
    MoveToPlayer(Vec<(i64, i64)>),
}

pub struct Enemy<'a> {
    pub character: Character<'a>,
    action: RefCell<EnemyAction>,
    start_x: i64,
    start_y: i64,
}

impl<'a> Enemy<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        texture: &'a Texture<'a>,
        surface: &'a Surface<'a>,
        x: i64,
        y: i64,
        id: Id,
        kind: CharacterKind,
    ) -> Enemy<'a> {
        let mut actions_standing = Vec::with_capacity(4);

        // up
        actions_standing.push(Dimension::new(Rect::new(0, 73, 28, 36), 0));
        // down
        actions_standing.push(Dimension::new(Rect::new(0, 3, 29, 37), 0));
        // left
        actions_standing.push(Dimension::new(Rect::new(0, 42, 37, 31), 0));
        // right
        actions_standing.push(Dimension::new(Rect::new(0, 115, 37, 31), 0));
        let mut actions_moving = Vec::with_capacity(4);
        actions_moving.push((Dimension::new(Rect::new(0, 73, 28, 36), 32), 1));
        actions_moving.push((Dimension::new(Rect::new(0, 3, 29, 37), 32), 1));
        actions_moving.push((Dimension::new(Rect::new(0, 42, 37, 31), 32), 1));
        actions_moving.push((Dimension::new(Rect::new(0, 115, 37, 31), 32), 1));

        // let texture = create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler =
            TextureHandler::new(surface, texture, actions_standing, actions_moving);

        Enemy {
            character: Character {
                action: Action {
                    direction: Direction::Down,
                    secondary: None,
                    movement: None,
                },
                x,
                y,
                health: Stat::new(1., 100),
                mana: Stat::new(1., 100),
                stamina: Stat::new(1., 100),
                xp_to_next_level: 1000,
                xp: 100,
                texture_handler,
                weapon: None,
                is_running: false,
                id,
                invincible_against: Vec::new(),
                statuses: Vec::new(),
                speed: ONE_SECOND / 60, // we want to move 60 times per second
                move_delay: 0,
                show_health_bar: true,
                death_animation: Some(DeathAnimation::new(texture_creator, ONE_SECOND)),
                kind,
            },
            action: RefCell::new(EnemyAction::None),
            start_x: x,
            start_y: y,
        }
    }

    fn compute_direction(&mut self, x: i64, y: i64) {
        let mut dir_x = None;
        let mut dir_y = None;
        if x > self.x() {
            dir_x = Some((Direction::Right, x - self.x()));
        } else if x < self.x() {
            dir_x = Some((Direction::Left, self.x() - x));
        }
        if y > self.y() {
            dir_y = Some((Direction::Down, y - self.y()));
        } else if y < self.y() {
            dir_y = Some((Direction::Up, self.y() - y));
        }
        match (dir_x, dir_y) {
            (Some((dir_x, distance_x)), Some((dir_y, distance_y))) => {
                if distance_x > distance_y {
                    self.character.action.direction = dir_x;
                    self.character.action.secondary = Some(dir_y);
                } else {
                    self.character.action.direction = dir_y;
                    self.character.action.secondary = Some(dir_x);
                }
            }
            (Some((dir_x, _)), None) => {
                self.character.action.direction = dir_x;
                self.character.action.secondary = None;
            }
            (None, Some((dir_y, _))) => {
                self.character.action.direction = dir_y;
                self.character.action.secondary = None;
            }
            (None, None) => {
                // We're "on" the player, which shouldn't be possible!
                self.character.action.secondary = None;
                *self.action.borrow_mut() = EnemyAction::None;
            }
        }
    }

    fn compute_adds(&self, target_x: i64, target_y: i64) -> (i64, i64) {
        println!("XXXX {} cmp {}", self.x(), target_x);
        println!("YYYY {} cmp {}", self.y(), target_y);
        (
            match self.x().cmp(&target_x) {
                Ordering::Less => 1,
                Ordering::Equal => 0,
                Ordering::Greater => -1,
            },
            match self.y().cmp(&target_y) {
                Ordering::Less => 1,
                Ordering::Equal => 0,
                Ordering::Greater => -1,
            },
        )
    }

    /// This method is used when we encountered an obstacle only!
    pub fn path_finder(
        &self,
        start_x: i64,
        start_y: i64,
        mut destination_x: i64,
        mut destination_y: i64,
        map: &Map,
        players: &[Player],
        npcs: &[Enemy],
    ) -> Option<Vec<(i64, i64)>> {
        destination_x -= destination_x % MAP_CASE_SIZE;
        destination_y -= destination_y % MAP_CASE_SIZE;

        let destination = (destination_x, destination_y);

        let mut closed_list = Vec::new();
        let mut open_list = BinaryHeap::new();
        let mut start_node = Node::new(
            start_x - start_x % MAP_CASE_SIZE,
            start_y - start_y % MAP_CASE_SIZE,
            0,
        );
        start_node.compute_heuristic(&destination);
        // Since we always want the node with the lowest heuristic at each turn,
        open_list.push(Reverse(start_node));

        let mut limit = 200;
        while let Some(node) = open_list.pop() {
            limit -= 1;
            if limit == 0 {
                break;
            }
            let node = node.0;
            if node.x != self.x() || node.y != self.y() {
                closed_list.push((node.x, node.y));
            }
            if node.x == destination_x && node.y == destination_y {
                // We're done!
                // We reverse the order so we go from the last node to the first one.
                closed_list.reverse();
                return Some(closed_list);
            } else {
                let nodes = vec![
                    (
                        Node::new(node.x + MAP_CASE_SIZE, node.y, node.cost + 1),
                        Direction::Right,
                    ),
                    (
                        Node::new(
                            node.x + MAP_CASE_SIZE,
                            node.y + MAP_CASE_SIZE,
                            node.cost + 1,
                        ),
                        Direction::Right,
                    ),
                    (
                        Node::new(node.x, node.y + MAP_CASE_SIZE, node.cost + 1),
                        Direction::Down,
                    ),
                    (
                        Node::new(
                            node.x - MAP_CASE_SIZE,
                            node.y + MAP_CASE_SIZE,
                            node.cost + 1,
                        ),
                        Direction::Left,
                    ),
                    (
                        Node::new(node.x - MAP_CASE_SIZE, node.y, node.cost + 1),
                        Direction::Left,
                    ),
                    (
                        Node::new(
                            node.x - MAP_CASE_SIZE,
                            node.y - MAP_CASE_SIZE,
                            node.cost + 1,
                        ),
                        Direction::Left,
                    ),
                    (
                        Node::new(node.x, node.y - MAP_CASE_SIZE, node.cost + 1),
                        Direction::Up,
                    ),
                    (
                        Node::new(
                            node.x + MAP_CASE_SIZE,
                            node.y - MAP_CASE_SIZE,
                            node.cost + 1,
                        ),
                        Direction::Right,
                    ),
                ]
                .into_iter()
                .filter_map(|(entry, dir)| {
                    if self
                        .character
                        .check_pos(dir, map, players, npcs, entry.x, entry.y)
                        && !closed_list
                            .iter()
                            .any(|entry2| entry.x == entry2.0 && entry.y == entry2.1)
                        && !open_list.iter().any(|entry2: &Reverse<Node>| {
                            entry == entry2.0 && entry.cost >= entry2.0.cost
                        })
                    {
                        Some(entry)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
                for mut node in nodes {
                    node.compute_heuristic(&destination);
                    open_list.push(Reverse(node));
                }
            }
        }
        None
    }

    pub fn apply_move(
        &self,
        map: &Map,
        elapsed: u64,
        players: &[Player],
        npcs: &[Enemy],
    ) -> (i64, i64) {
        let mut distance = utils::compute_distance(&players[0], self);
        let mut index = 0;
        // Would be nice to make two levels of detection:
        //  1. If the NPC sees a player (so a distance of ~50 meters)
        //  2. If the NPC hears a player (very close then)
        for (pos, player) in players.iter().enumerate().skip(1) {
            let tmp = utils::compute_distance(player, self);
            if tmp < distance {
                distance = tmp;
                index = pos;
            }
        }

        let min_target_dist = ::std::cmp::min(self.height(), self.width()) * 2;
        let new_action = match &*self.action.borrow() {
            EnemyAction::None | EnemyAction::MoveTo(..) if distance < min_target_dist as i32 => {
                if distance < 20 {
                    Some(EnemyAction::None)
                } else {
                    let player = &players[index];
                    // println!("Enemy is gonna chase player!");
                    if let Some(nodes) = self.path_finder(
                        self.x(),
                        self.y(),
                        player.x(),
                        player.y(),
                        map,
                        players,
                        npcs,
                    ) {
                        Some(EnemyAction::MoveToPlayer(nodes))
                    } else {
                        // We stop the movement to "watch" the enemy in case we can't reach it for
                        // whatever reason...
                        Some(EnemyAction::None)
                    }
                }
            }
            EnemyAction::None => {
                let mut x = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
                let mut y = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
                if x > -20 && x < 20 && y > -20 && y < 20 {
                    x = 20 * if x < 0 { -1 } else { 1 };
                    y = 20 * if y < 0 { -1 } else { 1 };
                }
                let mut x = x as i64 + self.start_x;
                let mut y = y as i64 + self.start_y;
                while !self.character.check_hitbox(
                    x - map.x,
                    y - map.y,
                    &map.data,
                    self.character.action.direction,
                ) {
                    x += 1;
                    y += 1;
                }
                if let Some(nodes) = self.path_finder(self.x(), self.y(), x, y, map, players, npcs)
                {
                    Some(EnemyAction::MoveTo(nodes))
                } else {
                    // Weird that no paths can reach the place, but whatever...
                    Some(EnemyAction::None)
                }
            }
            EnemyAction::MoveToPlayer(nodes) => {
                if distance > MAX_DISTANCE_PURSUIT {
                    // We stop going after this player.
                    if let Some(nodes) = self.path_finder(
                        self.x(),
                        self.y(),
                        self.start_x,
                        self.start_y,
                        map,
                        players,
                        npcs,
                    ) {
                        Some(EnemyAction::MoveTo(nodes))
                    } else {
                        Some(EnemyAction::None)
                    }
                } else if let Some(ref node) = nodes.first() {
                    if utils::compute_distance(node, &players[0]) > 30 {
                        // Player moved too much, we need to recompute a new path!
                        if let Some(nodes) = self.path_finder(
                            self.x(),
                            self.y(),
                            players[0].x(),
                            players[0].y(),
                            map,
                            players,
                            npcs,
                        ) {
                            Some(EnemyAction::MoveToPlayer(nodes))
                        } else {
                            // Weird that no paths can reach the place, but whatever...
                            None
                        }
                    } else {
                        let (target_x, target_y) = nodes[nodes.len() - 1];
                        let (x_add, y_add) = self.compute_adds(target_x, target_y);
                        if self
                            .character
                            .inner_check_move(map, players, npcs, x_add, y_add)
                            == (0, 0)
                        {
                            let (target_x, target_y) = nodes[0];
                            // If we encountered an unexpected obstacles? Let's recompute a path!
                            if let Some(nodes) = self.path_finder(
                                self.x(),
                                self.y(),
                                target_x,
                                target_y,
                                map,
                                players,
                                npcs,
                            ) {
                                Some(EnemyAction::MoveToPlayer(nodes))
                            } else {
                                // Weird that no path can reach the place, but whatever...
                                None
                            }
                        } else {
                            None
                        }
                    }
                } else {
                    Some(EnemyAction::None)
                }
            }
            EnemyAction::MoveTo(nodes) => {
                if nodes.is_empty() {
                    Some(EnemyAction::None)
                } else {
                    let (target_x, target_y) = nodes[nodes.len() - 1];
                    let (x_add, y_add) = self.compute_adds(target_x, target_y);
                    if self
                        .character
                        .inner_check_move(map, players, npcs, x_add, y_add)
                        == (0, 0)
                    {
                        let (target_x, target_y) = nodes[0];
                        // If we encountered an unexpected obstacles? Let's recompute a path!
                        if let Some(nodes) = self.path_finder(
                            self.x(),
                            self.y(),
                            target_x,
                            target_y,
                            map,
                            players,
                            npcs,
                        ) {
                            Some(EnemyAction::MoveTo(nodes))
                        } else {
                            // Weird that no path can reach the place, but whatever...
                            None
                        }
                    } else {
                        // Nothing to do here, just moving to the "target".
                        None
                    }
                }
            }
        };

        let mut action = self.action.borrow_mut();
        if let Some(new_action) = new_action {
            *action = new_action;
        }
        println!("next action: {:?}", action);
        // Time to apply actions now!
        match &mut *action {
            EnemyAction::None => (0, 0),
            EnemyAction::MoveTo(ref mut nodes) | EnemyAction::MoveToPlayer(ref mut nodes) => {
                if !nodes.is_empty()
                    && nodes[nodes.len() - 1].0 == self.x()
                    && nodes[nodes.len() - 1].1 == self.y()
                {
                    println!("POOOOOOP");
                    nodes.pop();
                }
                if let Some(ref node) = nodes.last() {
                    let (x_add, y_add) = self.compute_adds(node.0, node.1);
                    println!("---> ({}, {}) || ({}, {})", x_add, y_add, node.0, node.1);
                    self.character
                        .inner_check_move(map, players, npcs, x_add, y_add)
                } else {
                    (0, 0)
                }
            }
        }
    }

    pub fn update(&mut self, elapsed: u64, x: i64, y: i64) {
        if x > 0 {
            self.character.action.direction = Direction::Right;
        } else if x < 0 {
            self.character.action.direction = Direction::Left;
        }
        if y > 0 {
            self.character.action.direction = Direction::Down;
        } else if y < 0 {
            self.character.action.direction = Direction::Up;
        }
        if x != 0 || y != 0 {
            self.character.action.movement = Some(0);
        } else {
            self.character.action.movement = None;
        }
        println!("POS: ({}, {})", self.x(), self.y());
        self.character.update(elapsed, x, y)
    }
}

impl<'a> GetPos for Enemy<'a> {
    fn x(&self) -> i64 {
        self.character.x()
    }

    fn y(&self) -> i64 {
        self.character.y()
    }
}

impl<'a> GetDimension for Enemy<'a> {
    fn width(&self) -> u32 {
        self.character.width()
    }
    fn height(&self) -> u32 {
        self.character.height()
    }
}

impl<'a> Deref for Enemy<'a> {
    type Target = Character<'a>;

    fn deref(&self) -> &Self::Target {
        &self.character
    }
}

impl<'a> DerefMut for Enemy<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.character
    }
}
