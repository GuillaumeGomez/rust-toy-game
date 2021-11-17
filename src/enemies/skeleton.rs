use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::video::WindowContext;
use rand::Rng;

use crate::animation::Animation;
use crate::character::{Action, Character, CharacterKind, CharacterPoints, Direction, DirectionAndStrength, Obstacle};
use crate::enemy::{Enemy, EnemyAction};
use crate::map::Map;
use crate::player::Player;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::texture_holder::Textures;
use crate::utils;
use crate::weapons::Sword;
use crate::{
    GetDimension, GetPos, Id, FLOAT_COMPARISON_PRECISION, MAP_CASE_SIZE, MAX_DISTANCE_PURSUIT,
    MAX_DISTANCE_WANDERING, ONE_SECOND,
};

#[derive(Debug)]
struct Node {
    x: f32,
    y: f32,
    cost: u32,
    heuristic: u32,
}

impl Node {
    fn new(x: f32, y: f32, cost: u32) -> Node {
        Node {
            x,
            y,
            cost,
            heuristic: 0,
        }
    }
    fn compute_heuristic(&mut self, destination: &(f32, f32)) {
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
        (self.x - other.0.x).abs() < FLOAT_COMPARISON_PRECISION
            && (self.y - other.0.y).abs() < FLOAT_COMPARISON_PRECISION
    }
}
impl Eq for Node {}
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

pub struct Skeleton {
    pub character: Character,
    action: RefCell<EnemyAction>,
    start_x: f32,
    start_y: f32,
}

impl Skeleton {
    pub fn new<'a>(
        texture_creator: &TextureCreator<WindowContext>,
        textures: &Textures<'a>,
        x: f32,
        y: f32,
        id: Id,
        tile_width: u32,
        tile_height: u32,
    ) -> Self {
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
        let actions_moving = vec![
            // up
            (
                Dimension::new(Rect::new(0, 0, tile_width, tile_height), tile_width as i32),
                3,
            ),
            // down
            (
                Dimension::new(
                    Rect::new(0, tile_height as i32 * 3, tile_width, tile_height),
                    tile_width as i32,
                ),
                3,
            ),
            // left
            (
                Dimension::new(
                    Rect::new(0, tile_height as i32, tile_width, tile_height),
                    tile_width as i32,
                ),
                3,
            ),
            // right
            (
                Dimension::new(
                    Rect::new(0, tile_height as i32 * 2, tile_width, tile_height),
                    tile_width as i32,
                ),
                3,
            ),
        ];

        // let texture = create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler = TextureHandler::new(
            "skeleton",
            textures.get_texture_id_from_name("skeleton"),
            actions_standing,
            actions_moving,
            Some((24, 24)),
        );

        // CharacterStats {
        //     health: Stat::new(1., 100),
        //     mana: Stat::new(1., 100),
        //     stamina: Stat::new(10., 200),
        //     xp_to_next_level: 1000,
        //     xp: 100,
        //     level: 1,
        // }
        let level = 1;
        let points = CharacterPoints {
            strength: 1,
            constitution: 1,
            intelligence: 1,
            wisdom: 1,
            stamina: 1,
            agility: 1,
            dexterity: 1,
        };
        let stats = points.generate_stats(level);
        Self {
            character: Character {
                action: Action {
                    direction: DirectionAndStrength::new_with_strength(Direction::Down, 0.),
                    secondary: None,
                    movement: None,
                },
                x,
                y,
                level,
                points,
                stats,
                xp_to_next_level: 1000,
                xp: 100,
                unused_points: 0,
                texture_handler,
                weapon: Sword::new(textures, 10),
                is_running: false,
                id,
                invincible_against: Vec::new(),
                statuses: Vec::new(),
                speed: ONE_SECOND / 45, // we want to move 45 times per second
                move_delay: 0,
                tile_duration: ONE_SECOND / 8,
                tile_delay: 0,
                show_health_bar: true,
                death_animation: Some(Animation::new_death(textures)),
                kind: CharacterKind::Enemy,
                effect: RefCell::new(None),
                animations: Vec::new(),
                move_hitbox: (16, 8),
                blocking_direction: None,
                weapon_action: None,
            },
            action: RefCell::new(EnemyAction::None),
            start_x: x,
            start_y: y,
        }
    }

    fn compute_adds(&self, target_x: f32, target_y: f32) -> (f32, f32) {
        (
            match self.x().partial_cmp(&target_x).unwrap() {
                Ordering::Less => 1.,
                Ordering::Equal => 0.,
                Ordering::Greater => -1.,
            },
            match self.y().partial_cmp(&target_y).unwrap() {
                Ordering::Less => 1.,
                Ordering::Equal => 0.,
                Ordering::Greater => -1.,
            },
        )
    }

    fn get_directions(&self, x_add: f32, y_add: f32) -> (DirectionAndStrength, Option<DirectionAndStrength>) {
        if x_add != 0. && y_add != 0. {
            (
                if x_add > 0. {
                    DirectionAndStrength::new(Direction::Right)
                } else {
                    DirectionAndStrength::new(Direction::Left)
                },
                Some(if y_add > 0. {
                    DirectionAndStrength::new(Direction::Down)
                } else {
                    DirectionAndStrength::new(Direction::Up)
                }),
            )
        } else if x_add != 0. {
            (
                if x_add > 0. {
                    DirectionAndStrength::new(Direction::Right)
                } else {
                    DirectionAndStrength::new(Direction::Left)
                },
                None,
            )
        } else {
            (
                if y_add > 0. {
                    DirectionAndStrength::new(Direction::Down)
                } else {
                    DirectionAndStrength::new(Direction::Up)
                },
                None,
            )
        }
    }

    /// This method is used when we encountered an obstacle only!
    fn path_finder(
        &self,
        start_x: f32,
        start_y: f32,
        mut destination_x: f32,
        mut destination_y: f32,
        map: &Map,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
        step: f32,
        target_id: Option<Id>,
    ) -> Option<Vec<(f32, f32)>> {
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
                    Node::new(node.x + step, node.y, node.cost + 1),
                    Node::new(node.x + step, node.y + step, node.cost + 1),
                    Node::new(node.x, node.y + step, node.cost + 1),
                    Node::new(node.x - step, node.y + step, node.cost + 1),
                    Node::new(node.x - step, node.y, node.cost + 1),
                    Node::new(node.x - step, node.y - step, node.cost + 1),
                    Node::new(node.x, node.y - step, node.cost + 1),
                    Node::new(node.x + step, node.y - step, node.cost + 1),
                ]
                .into_iter()
                .filter(|entry| {
                    self.character
                        .check_map_pos(map, players, npcs, entry.x, entry.y, target_id)
                        == Obstacle::None
                        && !closed_list
                            .iter()
                            .any(|entry2| entry.x == entry2.0 && entry.y == entry2.1)
                        && !open_list.iter().any(|entry2: &Reverse<Node>| {
                            *entry == entry2.0 && entry.cost >= entry2.0.cost
                        })
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

    fn get_attack_direction<T: GetPos + GetDimension>(&self, target: &T) -> Direction {
        let dist_x = self.x() - target.x();
        let dist_y = self.y() - target.y();
        if dist_x.abs() > dist_y.abs() {
            if self.x() > target.x() {
                Direction::Left
            } else {
                Direction::Right
            }
        } else if self.y() < target.y() {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    fn move_back_from_target(
        &self,
        map: &Map,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
        self_x: f32,
        self_y: f32,
        target_x: f32,
        target_y: f32,
    ) -> Option<EnemyAction> {
        let mut res = None;
        if target_x > self_x {
            let (x_add, _) =
                self.character
                    .inner_check_move(map, players, npcs, DirectionAndStrength::new(Direction::Left), None, 0., 0.);
            if x_add != 0. {
                res = Some(EnemyAction::MoveTo(vec![(self_x - 1., self_y)]));
            }
        } else if target_x < self_x {
            let (x_add, _) =
                self.character
                    .inner_check_move(map, players, npcs, DirectionAndStrength::new(Direction::Right), None, 0., 0.);
            if x_add != 0. {
                res = Some(EnemyAction::MoveTo(vec![(self_x + 1., self_y)]));
            }
        }
        if res.is_none() {
            if target_y > self_y {
                let (_, y_add) = self.character.inner_check_move(
                    map,
                    players,
                    npcs,
                    DirectionAndStrength::new(Direction::Up),
                    None,
                    0.,
                    0.,
                );
                if y_add != 0. {
                    res = Some(EnemyAction::MoveTo(vec![(self_x, self_y - 1.)]));
                }
            } else if target_y < self_y {
                let (_, y_add) = self.character.inner_check_move(
                    map,
                    players,
                    npcs,
                    DirectionAndStrength::new(Direction::Down),
                    None,
                    0.,
                    0.,
                );
                if y_add != 0. {
                    res = Some(EnemyAction::MoveTo(vec![(self_x, self_y + 1.)]));
                }
            }
        }
        if res.is_none() {
            // Seems like we're stuck...
            Some(EnemyAction::None)
        } else {
            res
        }
    }
}

impl Enemy for Skeleton {
    #[inline]
    fn id(&self) -> Id {
        self.character.id
    }
    #[inline]
    fn character(&self) -> &Character {
        &self.character
    }
    #[inline]
    fn character_mut(&mut self) -> &mut Character {
        unsafe { std::mem::transmute(&mut self.character) }
    }

    fn update(&mut self, elapsed: u32, x: f32, y: f32) {
        if x > 0. {
            self.character.action.direction = DirectionAndStrength::new(Direction::Right);
        } else if x < 0. {
            self.character.action.direction = DirectionAndStrength::new(Direction::Left);
        } else if y > 0. {
            self.character.action.direction = DirectionAndStrength::new(Direction::Down);
        } else if y < 0. {
            self.character.action.direction = DirectionAndStrength::new(Direction::Up);
        }
        if x != 0. || y != 0. {
            if self.character.action.movement.is_none() {
                self.character.action.movement = Some(0);
            }
        } else {
            self.character.action.movement = None;
        }
        if !self.character.is_attacking() && self.action.borrow().is_attack() {
            self.character.attack();
            if x == 0. && y == 0. {
                match &*self.action.borrow() {
                    EnemyAction::Attack(ref dir) => self.character.action.direction.direction = *dir,
                    _ => {}
                }
            }
        }
        self.character.update(elapsed, x, y, None)
    }

    fn apply_move(
        &self,
        map: &Map,
        _elapsed: u32,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
    ) -> (f32, f32) {
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

        let wander_target_distance =
            utils::compute_distance(&players[index], &(self.start_x, self.start_y));

        let self_x = self.x();
        let self_y = self.y();

        debug_enemy!(
            "[{}] Distance to player: {} / {}, wander: {} / {}, pos: ({}, {})",
            self.id,
            distance,
            crate::ONE_METER as i32 * 8,
            wander_target_distance,
            MAX_DISTANCE_WANDERING,
            self_x,
            self_y,
        );

        let weapon_height = self.character.weapon.height() * 3 / 4;

        let new_action = match &mut *self.action.borrow_mut() {
            EnemyAction::None
            | EnemyAction::MoveTo(..)
            | EnemyAction::MoveToPlayer(..)
            | EnemyAction::Attack(_)
                if (distance as u32) < weapon_height + MAP_CASE_SIZE as u32
                    && wander_target_distance < MAX_DISTANCE_WANDERING as f32 =>
            {
                // We're in attack range, however we need to check if we're not in a corner (which
                // would prevent the attack to work!).
                let target = &players[index];
                let target_x = target.x();
                let target_y = target.y();
                let dist_x = (self_x - target_x).abs() as u32;
                let dist_y = (self_y - target_y).abs() as u32;

                // Little explanations here: we first try to get on the same axis than the user to
                // be able to attack him. If we can't, then we find a way to the target by creating
                // a path.
                if dist_x > weapon_height / 3 && dist_y > weapon_height / 2 {
                    debug_enemy!("[{}] Re-adjusting position v1!", self.id);
                    if self
                        .character
                        .check_map_pos(map, players, npcs, target_x, self_y, None)
                        == Obstacle::None
                    {
                        debug_enemy!("[{}] we can move to player (on x)!", self.id);
                        Some(EnemyAction::MoveToPlayer(vec![(target_x, self_y)]))
                    } else if self
                        .character
                        .check_map_pos(map, players, npcs, self_x, target_y, None)
                        == Obstacle::None
                    {
                        debug_enemy!("[{}] we can move to player (on y)!", self.id);
                        Some(EnemyAction::MoveToPlayer(vec![(self_x, target_y)]))
                    } else {
                        debug_enemy!("[{}] move back to move closer!", self.id);
                        // We seem to not be able to move closer to the target, let's try to move
                        // around then!
                        self.move_back_from_target(
                            map, players, npcs, self_x, self_y, target_x, target_y,
                        )
                    }
                } else {
                    // Little explanations here: we first try to get on the same axis than the user to
                    // be able to attack him. If we can't, then we find a way to the target by creating
                    // a path.
                    if distance >= weapon_height as f32 {
                        debug_enemy!("[{}] Re-adjusting position v2!", self.id);
                        Some(EnemyAction::MoveToPlayer(vec![(
                            self_x
                                + if self_x > target_x {
                                    MAP_CASE_SIZE * -1
                                } else if self_x < target_x {
                                    MAP_CASE_SIZE
                                } else {
                                    0
                                } as f32, // + incr_x as f32 * MAP_CASE_SIZE,
                            self_y
                                + if self_y > target_y {
                                    MAP_CASE_SIZE * -1
                                } else if self_y < target_y {
                                    MAP_CASE_SIZE
                                } else {
                                    0
                                } as f32, // + incr_y as f32 * MAP_CASE_SIZE,
                        )]))
                    } else {
                        debug_enemy!("[{}] attacking!", self.id);
                        Some(EnemyAction::Attack(self.get_attack_direction(target)))
                    }
                }
            }
            EnemyAction::None | EnemyAction::MoveTo(..)
                if distance < (crate::ONE_METER * 8) as f32
                    && wander_target_distance < MAX_DISTANCE_WANDERING as f32 =>
            {
                let player = &players[index];
                // debug_enemy!("Enemy is gonna chase player!");
                if let Some(nodes) = self.path_finder(
                    self_x,
                    self_y,
                    player.x(),
                    player.y(),
                    map,
                    &players,
                    npcs,
                    MAP_CASE_SIZE as _,
                    // We exclude the "target" to allow the path finder to not be able to finish
                    Some(player.id),
                ) {
                    debug_enemy!("[{}] Moving to player {:?}", self.id, nodes);
                    Some(EnemyAction::MoveToPlayer(nodes))
                } else {
                    // We stop the movement to "watch" the enemy in case we can't reach it for
                    // whatever reason...
                    Some(EnemyAction::None)
                }
            }
            EnemyAction::None | EnemyAction::Attack(_) => {
                let mut x = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
                let mut y = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
                if x > -20 && x < 20 && y > -20 && y < 20 {
                    x = 20 * if x < 0 { -1 } else { 1 };
                    y = 20 * if y < 0 { -1 } else { 1 };
                }
                let mut x = x as f32 + self.start_x;
                let mut y = y as f32 + self.start_y;
                while !self.character.check_hitbox(
                    x - map.x,
                    y - map.y,
                    &map.data,
                    *self.character.action.direction,
                ) {
                    x += 1.;
                    y += 1.;
                }
                if let Some(nodes) = self.path_finder(
                    self_x,
                    self_y,
                    x,
                    y,
                    map,
                    players,
                    npcs,
                    MAP_CASE_SIZE as _,
                    None,
                ) {
                    Some(EnemyAction::MoveTo(nodes))
                } else {
                    // Weird that no paths can reach the place, but whatever...
                    Some(EnemyAction::None)
                }
            }
            EnemyAction::MoveToPlayer(nodes) => {
                if distance > MAX_DISTANCE_PURSUIT as f32
                    || utils::compute_distance(&(self.start_x, self.start_y), self)
                        > MAX_DISTANCE_WANDERING as f32
                {
                    // We stop going after this player.
                    if let Some(nodes) = self.path_finder(
                        self_x,
                        self_y,
                        self.start_x,
                        self.start_y,
                        map,
                        players,
                        npcs,
                        MAP_CASE_SIZE as _,
                        None,
                    ) {
                        Some(EnemyAction::MoveTo(nodes))
                    } else {
                        Some(EnemyAction::None)
                    }
                } else if let Some(ref node) = nodes.first() {
                    let player = &players[index];
                    if utils::compute_distance(node, player) > (crate::ONE_METER * 2) as f32 {
                        // Player moved too much, we need to recompute a new path!
                        if let Some(nodes) = self.path_finder(
                            self_x,
                            self_y,
                            player.x(),
                            player.y(),
                            map,
                            // We exclude the "target" to allow the path finder to not be able to finish
                            players,
                            npcs,
                            MAP_CASE_SIZE as _,
                            Some(player.id),
                        ) {
                            debug_enemy!("[{}] recomputed path to player: {:?}", self.id, nodes);
                            Some(EnemyAction::MoveToPlayer(nodes))
                        } else {
                            // Weird that no paths can reach the place, but whatever...
                            None
                        }
                    } else {
                        let (target_x, target_y) = nodes[nodes.len() - 1];
                        let (x_add, y_add) = self.compute_adds(target_x, target_y);
                        match self.character.check_map_pos(
                            map,
                            players,
                            npcs,
                            self_x + x_add,
                            self_y + y_add,
                            None,
                        ) {
                            Obstacle::Map => {
                                if nodes.len() > 1 && !(x_add != 0. && y_add != 0.) {
                                    let (next_x, next_y) = nodes[nodes.len() - 2];
                                    let pos = nodes.len() - 1;
                                    if x_add != 0. {
                                        if next_y > self_y {
                                            nodes[pos].1 = self_y + MAP_CASE_SIZE as f32;
                                        } else {
                                            nodes[pos].1 = self_y - MAP_CASE_SIZE as f32;
                                        }
                                    } else {
                                        if next_x > self_x {
                                            nodes[pos].0 = self_x + MAP_CASE_SIZE as f32;
                                        } else {
                                            nodes[pos].0 = self_x - MAP_CASE_SIZE as f32;
                                        }
                                    }
                                    None
                                } else {
                                    Some(EnemyAction::None)
                                }
                            }
                            _ => None,
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
                    match self.character.check_map_pos(
                        map,
                        players,
                        npcs,
                        self_x + x_add,
                        self_y + y_add,
                        None,
                    ) {
                        Obstacle::Map => {
                            if nodes.len() > 1 && !(x_add != 0. && y_add != 0.) {
                                let (next_x, next_y) = nodes[nodes.len() - 2];
                                let pos = nodes.len() - 1;
                                if x_add != 0. {
                                    if next_y > self_y {
                                        nodes[pos].1 = self_y + MAP_CASE_SIZE as f32;
                                    } else {
                                        nodes[pos].1 = self_y - MAP_CASE_SIZE as f32;
                                    }
                                } else {
                                    if next_x > self_x {
                                        nodes[pos].0 = self_x + MAP_CASE_SIZE as f32;
                                    } else {
                                        nodes[pos].0 = self_x - MAP_CASE_SIZE as f32;
                                    }
                                }
                                None
                            } else {
                                Some(EnemyAction::None)
                            }
                        }
                        Obstacle::Character => {
                            debug_enemy!("[{}] character in the path", self.id);
                            // We need to recompute the path
                            if let Some(nodes) = self.path_finder(
                                self_x,
                                self_y,
                                nodes[0].0,
                                nodes[0].1,
                                map,
                                players,
                                npcs,
                                MAP_CASE_SIZE as _,
                                None,
                            ) {
                                Some(EnemyAction::MoveTo(nodes))
                            } else {
                                // Weird that no path can reach the place, but whatever...
                                None
                            }
                        }
                        _ => {
                            debug_enemy!("[{}] no problem with the path apparently!", self.id);
                            None
                        }
                    }
                }
            }
        };

        let mut action = self.action.borrow_mut();
        debug_enemy!(
            "[{}] next action: {:?}{}",
            self.id,
            if let Some(ref new_action) = new_action {
                new_action
            } else {
                &*action
            },
            if new_action.is_some() {
                " from new action"
            } else {
                ""
            }
        );
        if let Some(new_action) = new_action {
            *action = new_action;
        }
        // Time to apply actions now!
        match &mut *action {
            EnemyAction::None | EnemyAction::Attack(_) => (0., 0.),
            EnemyAction::MoveTo(ref mut nodes) | EnemyAction::MoveToPlayer(ref mut nodes) => {
                if !nodes.is_empty()
                    && nodes[nodes.len() - 1].0 == self_x
                    && nodes[nodes.len() - 1].1 == self_y
                {
                    nodes.pop();
                }
                if !nodes.is_empty() {
                    let node = &nodes[nodes.len() - 1];
                    let (x_add, y_add) = self.compute_adds(node.0, node.1);
                    let (dir, dir2) = self.get_directions(x_add, y_add);
                    debug_enemy!(
                        "[{}] ---> [{:?}] ({}, {}) || ({}, {}) => ({}, {})",
                        self.id,
                        dir,
                        x_add,
                        y_add,
                        self_x,
                        self_y,
                        node.0,
                        node.1
                    );
                    let (x_add, y_add) = self
                        .character
                        .inner_check_move(map, players, npcs, dir, dir2, 0., 0.);
                    if x_add == 0. && y_add == 0. {
                        debug_enemy!("[{}] Path blocked, forcing recomputation!", self.id);
                        nodes.clear();
                    }
                    (x_add, y_add)
                } else {
                    (0., 0.)
                }
            }
        }
    }

    fn draw(&mut self, system: &mut crate::system::System, debug: bool) {
        use crate::sdl2::rect::Point;
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
