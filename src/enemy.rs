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

#[derive(Clone)]
struct MoveNode {
    x: i64,
    y: i64,
    direction: Direction,
}

// TODO: for moveto and movetoplayer, add "nodes" after a little path finding to go around obstacles
#[derive(Clone)]
enum EnemyAction {
    // Not doing anything for the moment...
    None,
    MoveTo(Vec<MoveNode>),
    // Targetted player (in case of multiplayer, might be nice to have IDs for players)
    MoveToPlayer,
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

    fn compute_destination(&mut self, x: i64, y: i64) {
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
    ) -> Vec<(i64, i64)> {
        destination_x -= destination_x % MAP_CASE_SIZE;
        destination_y -= destination_y % MAP_CASE_SIZE;

        let destination = (destination_x, destination_y);

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
                self.heuristic =
                    utils::compute_distance(&(self.x, self.y), destination) as u32 + self.cost;
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

        let mut x = 0;
        println!("destination: ({}, {})", destination.0, destination.1);
        println!("destination: ({}, {})", destination_x, destination_y);
        while let Some(node) = open_list.pop() {
            x += 1;
            if x >= 50 {
                panic!("fuck");
            }
            let node = node.0;
            println!("===> current node: {:?}", node);
            if node.x == destination_x && node.y == destination_y {
                // done!
                println!("===> {:#?}", closed_list);
                panic!("yolo");
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
                    if self.check_pos(dir, map, players, npcs, entry.x, entry.y)
                        && !open_list.iter().any(|entry2| entry == *entry2)
                        && !closed_list
                            .iter()
                            .any(|entry2: &Node| entry == *entry2 && entry.cost >= entry2.cost)
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
            closed_list.push(node);
        }
        vec![]
    }

    // pub fn apply_move(
    //     &self,
    //     map: &Map,
    //     elapsed: u64,
    //     players: &[Player],
    //     npcs: &[Enemy],
    // ) -> (i64, i64) {
    //     let mut distance = utils::compute_distance(&players[0], self);
    //     for player in players.iter() {
    //         let tmp = utils::compute_distance(player, self);
    //         if tmp < distance {
    //             distance = tmp;
    //         }
    //     }
    //     loop {
    //         match &*self.action.borrow() {
    //             EnemyAction::None | EnemyAction::MoveTo(..)
    //                 if distance < (::std::cmp::min(self.height(), self.width()) * 2) as i32 =>
    //             {
    //                 // println!("Enemy is gonna chase player!");
    //                 *self.action.borrow_mut() = EnemyAction::MoveToPlayer;
    //             }
    //             EnemyAction::None => {
    //                 let mut x = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
    //                 let mut y = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
    //                 if x > -20 && x < 20 && y > -20 && y < 20 {
    //                     x = 20 * if x < 0 { -1 } else { 1 };
    //                     y = 20 * if y < 0 { -1 } else { 1 };
    //                 }
    //                 *self.action.borrow_mut() = EnemyAction::MoveTo(vec![(
    //                     x as i64 + self.start_x,
    //                     y as i64 + self.start_y,
    //                 )]);
    //                 // println!(
    //                 //     "Enemy is gonna move to ({} {})",
    //                 //     x + self.start_x,
    //                 //     y + self.start_y
    //                 // );
    //                 // self.character.borrow_mut().action.movement = Some(0);
    //             }
    //             EnemyAction::MoveTo(nodes) => {
    //                 let node = &nodes[nodes.len() - 1];
    //                 if node.0 == self.x() && node.1 == self.y() {
    //                     // println!("Enemy reached destination!");
    //                     // We reached the goal, let's find another one. :)
    //                     // nodes.pop();
    //                     if nodes.is_empty() {
    //                         *self.action.borrow_mut() = EnemyAction::None;
    //                     // self.character.borrow_mut().action.movement = None;
    //                     } else {
    //                         continue;
    //                     }
    //                 } else {
    //                     // self.compute_destination(x, y);
    //                     if self.inner_apply_move(map, players, npcs, 0, 0) == (0, 0) {
    //                         // println!("Enemy cannot move there");
    //                         *self.action.borrow_mut() = EnemyAction::None;
    //                     // self.character.borrow_mut().action.movement = None;
    //                     } else {
    //                         // self.character.borrow_mut().action.movement = Some(0);
    //                         // println!("Enemy is moving");
    //                     }
    //                 }
    //             }
    //             EnemyAction::MoveToPlayer => {
    //                 if distance > MAX_DISTANCE_PURSUIT {
    //                     // println!("Enemy stop chasing player (player too far)");
    //                     // We come back to the initial position
    //                     *self.action.borrow_mut() =
    //                         EnemyAction::MoveTo(vec![(self.start_x, self.start_y)]);
    //                 // self.character.borrow_mut().action.movement = None;
    //                 } else if distance < players[0].width() as i32 + 6
    //                     || distance < players[0].height() as i32 + 6
    //                 {
    //                     // println!("Enemy stop chasing player (reached player)");
    //                     *self.action.borrow_mut() = EnemyAction::None;
    //                 // self.character.borrow_mut().action.movement = None;
    //                 } else {
    //                     // println!("Enemy chasing player");
    //                     // self.compute_destination(players[0].x(), players[0].y());
    //                     // self.character.borrow_mut().action.movement = Some(0);
    //                     // self.character.borrow_mut().inner_apply_move(map, players, npcs, 0, 0);
    //                 }
    //             }
    //         }
    //         break;
    //     }
    //     (0, 0)
    // }
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
        self.width()
    }
    fn height(&self) -> u32 {
        self.height()
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
