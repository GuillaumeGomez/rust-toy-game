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

use crate::character::{Action, Character, CharacterKind, Direction, Obstacle};
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
        // // up
        // actions_standing.push(Dimension::new(Rect::new(0, 73, 28, 36), 0));
        // // down
        // actions_standing.push(Dimension::new(Rect::new(0, 3, 29, 37), 0));
        // // left
        // actions_standing.push(Dimension::new(Rect::new(0, 42, 37, 31), 0));
        // // right
        // actions_standing.push(Dimension::new(Rect::new(0, 115, 37, 31), 0));
        // let mut actions_moving = Vec::with_capacity(4);
        // actions_moving.push((Dimension::new(Rect::new(0, 73, 28, 36), 32), 1));
        // actions_moving.push((Dimension::new(Rect::new(0, 3, 29, 37), 32), 1));
        // actions_moving.push((Dimension::new(Rect::new(0, 42, 37, 31), 32), 1));
        // actions_moving.push((Dimension::new(Rect::new(0, 115, 37, 31), 32), 1));

        let tile_width = surface.width() / 3;
        let tile_height = surface.height() / 4;
        let actions_standing = vec![
            // up
            Dimension::new(Rect::new(tile_width as i32, 0, tile_width, tile_height), 0),
            // down
            Dimension::new(
                Rect::new(
                    tile_width as i32,
                    tile_height as i32 * 3,
                    tile_width,
                    tile_height,
                ),
                0,
            ),
            // left
            Dimension::new(
                Rect::new(
                    tile_width as i32,
                    tile_height as i32,
                    tile_width,
                    tile_height,
                ),
                0,
            ),
            // right
            Dimension::new(
                Rect::new(
                    tile_width as i32,
                    tile_height as i32 * 2,
                    tile_width,
                    tile_height,
                ),
                0,
            ),
        ];
        let mut actions_moving = vec![
            (
                Dimension::new(
                    Rect::new(tile_width as i32, 0, tile_width, tile_height),
                    tile_width as i32,
                ),
                3,
            ),
            (
                Dimension::new(
                    Rect::new(
                        tile_width as i32,
                        tile_height as i32 * 3,
                        tile_width,
                        tile_height,
                    ),
                    tile_width as i32,
                ),
                3,
            ),
            (
                Dimension::new(
                    Rect::new(
                        tile_width as i32,
                        tile_height as i32,
                        tile_width,
                        tile_height,
                    ),
                    tile_width as i32,
                ),
                3,
            ),
            (
                Dimension::new(
                    Rect::new(
                        tile_width as i32,
                        tile_height as i32 * 2,
                        tile_width,
                        tile_height,
                    ),
                    tile_width as i32,
                ),
                3,
            ),
        ];

        // let texture = create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler = TextureHandler::new(
            surface,
            texture,
            actions_standing,
            actions_moving,
            Some((24, 24)),
        );

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

    fn get_directions(&self, x_add: i64, y_add: i64) -> (Direction, Option<Direction>) {
        if x_add != 0 && y_add != 0 {
            (
                if x_add > 0 {
                    Direction::Right
                } else {
                    Direction::Left
                },
                Some(if y_add > 0 {
                    Direction::Down
                } else {
                    Direction::Up
                }),
            )
        } else if x_add != 0 {
            (
                if x_add > 0 {
                    Direction::Right
                } else {
                    Direction::Left
                },
                None,
            )
        } else {
            (
                if y_add > 0 {
                    Direction::Down
                } else {
                    Direction::Up
                },
                None,
            )
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
        step: i64,
        target_id: Option<Id>,
    ) -> Option<Vec<(i64, i64)>> {
        destination_x -= destination_x % step;
        destination_y -= destination_y % step;

        let destination = (destination_x, destination_y);

        let mut closed_list = Vec::new();
        let mut open_list = BinaryHeap::new();
        let mut start_node = Node::new(start_x - start_x % step, start_y - start_y % step, 0);
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
                        Node::new(node.x + step, node.y, node.cost + 1),
                        [Direction::Right].into_iter(),
                    ),
                    (
                        Node::new(node.x + step, node.y + step, node.cost + 1),
                        [Direction::Right, Direction::Down].into_iter(),
                    ),
                    (
                        Node::new(node.x, node.y + step, node.cost + 1),
                        [Direction::Down].into_iter(),
                    ),
                    (
                        Node::new(node.x - step, node.y + step, node.cost + 1),
                        [Direction::Left, Direction::Down].into_iter(),
                    ),
                    (
                        Node::new(node.x - step, node.y, node.cost + 1),
                        [Direction::Left].into_iter(),
                    ),
                    (
                        Node::new(node.x - step, node.y - step, node.cost + 1),
                        [Direction::Left, Direction::Up].into_iter(),
                    ),
                    (
                        Node::new(node.x, node.y - step, node.cost + 1),
                        [Direction::Up].into_iter(),
                    ),
                    (
                        Node::new(node.x + step, node.y - step, node.cost + 1),
                        [Direction::Right, Direction::Up].into_iter(),
                    ),
                ]
                .into_iter()
                .filter_map(|(entry, mut directions)| {
                    if directions.all(|dir| {
                        self.character
                            .check_map_pos(*dir, map, players, npcs, entry.x, entry.y, target_id)
                            == Obstacle::None
                    }) && !closed_list
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
        let new_action = match &mut *self.action.borrow_mut() {
            EnemyAction::None | EnemyAction::MoveTo(..)
                if distance < crate::ONE_METER as i32 * 8 =>
            {
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
                        &players,
                        npcs,
                        MAP_CASE_SIZE,
                        // We exclude the "target" to allow the path finder to not be able to finish
                        Some(player.id),
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
                if let Some(nodes) = self.path_finder(
                    self.x(),
                    self.y(),
                    x,
                    y,
                    map,
                    players,
                    npcs,
                    MAP_CASE_SIZE,
                    None,
                ) {
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
                        MAP_CASE_SIZE,
                        None,
                    ) {
                        Some(EnemyAction::MoveTo(nodes))
                    } else {
                        Some(EnemyAction::None)
                    }
                } else if let Some(ref node) = nodes.first() {
                    if utils::compute_distance(node, &players[0]) > crate::ONE_METER as i32 * 2 {
                        let player = &players[0];
                        // Player moved too much, we need to recompute a new path!
                        if let Some(nodes) = self.path_finder(
                            self.x(),
                            self.y(),
                            player.x(),
                            player.y(),
                            map,
                            // We exclude the "target" to allow the path finder to not be able to finish
                            &players,
                            npcs,
                            MAP_CASE_SIZE,
                            Some(player.id),
                        ) {
                            Some(EnemyAction::MoveToPlayer(nodes))
                        } else {
                            // Weird that no paths can reach the place, but whatever...
                            None
                        }
                    } else {
                        let (target_x, target_y) = nodes[nodes.len() - 1];
                        let (x_add, y_add) = self.compute_adds(target_x, target_y);
                        let (dir, dir2) = self.get_directions(x_add, y_add);
                        match self.character.check_map_pos(
                            dir,
                            map,
                            players,
                            npcs,
                            self.x() + x_add,
                            self.y() + y_add,
                            None,
                        ) {
                            Obstacle::Map => {
                                if nodes.len() > 1 && !(x_add != 0 && y_add != 0) {
                                    let (next_x, next_y) = nodes[nodes.len() - 2];
                                    let pos = nodes.len() - 1;
                                    if x_add != 0 {
                                        if next_y > self.y() {
                                            nodes[pos].1 = self.y() + MAP_CASE_SIZE;
                                        } else {
                                            nodes[pos].1 = self.y() - MAP_CASE_SIZE;
                                        }
                                    } else {
                                        if next_x > self.x() {
                                            nodes[pos].0 = self.x() + MAP_CASE_SIZE;
                                        } else {
                                            nodes[pos].0 = self.x() - MAP_CASE_SIZE;
                                        }
                                    }
                                    None
                                } else {
                                    Some(EnemyAction::None)
                                }
                            }
                            _ => None,
                        }
                        // let (target_x, target_y) = nodes[nodes.len() - 1];
                        // let (x_add, y_add) = self.compute_adds(target_x, target_y);
                        // let (dir, dir2) = self.get_directions(x_add, y_add);
                        // if self
                        //     .character
                        //     .inner_check_move(map, players, npcs, x_add, y_add, dir, dir2)
                        //     == (0, 0)
                        // {
                        // panic!("map obstacle 1");
                        // If we encountered an unexpected obstacles? Let's recompute a path!
                        // if let Some(extra_nodes) = self.path_finder(
                        //     self.x(),
                        //     self.y(),
                        //     target_x,
                        //     target_y,
                        //     map,
                        //     players,
                        //     npcs,
                        //     1,
                        // ) {
                        //     let mut nodes = nodes.clone();
                        //     nodes.pop(); // we remove the unneeded last node
                        //     nodes.extend(extra_nodes.into_iter());
                        //     Some(EnemyAction::MoveToPlayer(nodes))
                        // } else {
                        // Weird that no path can reach the place, but whatever...
                        // None
                        // }
                        // } else {
                        //     None
                        // }
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
                    let (dir, dir2) = self.get_directions(x_add, y_add);
                    match self.character.check_map_pos(
                        dir,
                        map,
                        players,
                        npcs,
                        self.x() + x_add,
                        self.y() + y_add,
                        None,
                    ) {
                        Obstacle::Map => {
                            if nodes.len() > 1 && !(x_add != 0 && y_add != 0) {
                                let (next_x, next_y) = nodes[nodes.len() - 2];
                                let pos = nodes.len() - 1;
                                if x_add != 0 {
                                    if next_y > self.y() {
                                        nodes[pos].1 = self.y() + MAP_CASE_SIZE;
                                    } else {
                                        nodes[pos].1 = self.y() - MAP_CASE_SIZE;
                                    }
                                } else {
                                    if next_x > self.x() {
                                        nodes[pos].0 = self.x() + MAP_CASE_SIZE;
                                    } else {
                                        nodes[pos].0 = self.x() - MAP_CASE_SIZE;
                                    }
                                }
                                None
                            } else {
                                Some(EnemyAction::None)
                            }
                            // panic!("map obstacle 2");
                            // If we encountered an unexpected obstacles? Let's recompute a path!
                            // println!("NEED TO GO: ({}, {}) => ({}, {})", self.x(), self.y(), target_x, target_y);
                            // if let Some(extra_nodes) = self.path_finder(
                            //     self.x(),
                            //     self.y(),
                            //     target_x,
                            //     target_y,
                            //     map,
                            //     players,
                            //     npcs,
                            //     1,
                            // ) {
                            //     let mut nodes = nodes.clone();
                            //     nodes.pop(); // we remove the unneeded last node
                            //     nodes.extend(extra_nodes.into_iter());
                            //     Some(EnemyAction::MoveTo(nodes))
                            // } else {
                            // Weird that no path can reach the place, but whatever...
                            // None
                            // }
                        }
                        Obstacle::Character => {
                            println!("character in the path");
                            // We need to recompute the path
                            if let Some(nodes) = self.path_finder(
                                self.x(),
                                self.y(),
                                nodes[0].0,
                                nodes[0].1,
                                map,
                                players,
                                npcs,
                                MAP_CASE_SIZE,
                                None,
                            ) {
                                Some(EnemyAction::MoveTo(nodes))
                            } else {
                                // Weird that no path can reach the place, but whatever...
                                None
                            }
                        }
                        _ => {
                            println!("no problem with the path apparently!");
                            None
                        }
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
                    nodes.pop();
                }
                if !nodes.is_empty() {
                    let node = &nodes[nodes.len() - 1];
                    let (x_add, y_add) = self.compute_adds(node.0, node.1);
                    let (dir, dir2) = self.get_directions(x_add, y_add);
                    println!(
                        "---> [{:?}] ({}, {}) || ({}, {}) => ({}, {})",
                        dir,
                        x_add,
                        y_add,
                        self.x(),
                        self.y(),
                        node.0,
                        node.1
                    );
                    self.character
                        .inner_check_move(map, players, npcs, dir, dir2)
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
        } else if y > 0 {
            self.character.action.direction = Direction::Down;
        } else if y < 0 {
            self.character.action.direction = Direction::Up;
        }
        if x != 0 || y != 0 {
            self.character.action.movement = Some(0);
        } else {
            self.character.action.movement = None;
        }
        self.character.update(elapsed, x, y)
    }

    pub fn draw(&mut self, system: &mut crate::system::System, debug: bool) {
        use sdl2::rect::Point;
        if debug {
            match &*self.action.borrow() {
                EnemyAction::MoveTo(ref nodes) | EnemyAction::MoveToPlayer(ref nodes) => {
                    let mut iter = nodes.iter().peekable();
                    while let Some(node) = iter.next() {
                        if let Some(next) = iter.peek() {
                            system
                                .canvas
                                .draw_line(
                                    Point::new(
                                        (next.0 - system.x()) as i32,
                                        (next.1 - system.y()) as i32,
                                    ),
                                    Point::new(
                                        (node.0 - system.x()) as i32,
                                        (node.1 - system.y()) as i32,
                                    ),
                                )
                                .unwrap();
                        } else {
                            system
                                .canvas
                                .draw_line(
                                    Point::new(
                                        (self.x() - system.x()) as i32,
                                        (self.y() - system.y()) as i32,
                                    ),
                                    Point::new(
                                        (node.0 - system.x()) as i32,
                                        (node.1 - system.y()) as i32,
                                    ),
                                )
                                .unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        self.character.draw(system, debug);
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
