use rand::Rng;
use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::character::{Action, Character, Direction};
use crate::map::Map;
use crate::player::Player;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::utils;
use crate::{GetDimension, GetPos, MAX_DISTANCE_PURSUIT, MAX_DISTANCE_WANDERING};

// TODO: for moveto and movetoplayer, add "nodes" after a little path finding to go around obstacles
#[derive(Clone, Copy)]
enum EnemyAction {
    // Not doing anything for the moment...
    None,
    // MoveTo(x, y)
    MoveTo(i32, i32),
    // Targetted player (in case of multiplayer, might be nice to have IDs for players)
    MoveToPlayer,
}

pub struct Enemy<'a> {
    character: Character<'a>,
    action: EnemyAction,
    start_x: i32,
    start_y: i32,
}

impl<'a> Enemy<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, x: i32, y: i32) -> Enemy<'a> {
        let mut actions_standing = Vec::with_capacity(4);

        // front
        actions_standing.push(Dimension::new(Rect::new(0, 73, 28, 36), 0));
        // left
        actions_standing.push(Dimension::new(Rect::new(0, 42, 37, 31), 0));
        // right
        actions_standing.push(Dimension::new(Rect::new(0, 115, 37, 31), 0));
        // back
        actions_standing.push(Dimension::new(Rect::new(0, 3, 29, 37), 0));
        let mut actions_moving = Vec::with_capacity(4);
        actions_moving.push((Dimension::new(Rect::new(0, 73, 28, 36), 32), 1));
        actions_moving.push((Dimension::new(Rect::new(0, 42, 37, 31), 32), 1));
        actions_moving.push((Dimension::new(Rect::new(0, 115, 37, 31), 32), 1));
        actions_moving.push((Dimension::new(Rect::new(0, 3, 29, 37), 32), 1));

        let surface = Surface::from_file("resources/enemy1.png")
            .expect("failed to load `resources/enemy1.png`");

        let texture = texture_creator
            .create_texture_from_surface(surface)
            .expect("failed to build texture from surface");
        // let texture = create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler = TextureHandler {
            texture,
            actions_standing,
            actions_moving,
        };

        Enemy {
            character: Character {
                action: Action {
                    direction: Direction::Front,
                    secondary: None,
                    movement: None,
                },
                x,
                y,
                total_health: 100,
                health: 75,
                total_mana: 100,
                mana: 20,
                total_stamina: 100,
                stamina: 100,
                xp_to_next_level: 1000,
                xp: 150,
                texture_handler,
            },
            action: EnemyAction::None,
            start_x: x,
            start_y: y,
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        self.character.draw(canvas, false, screen)
    }

    fn compute_destination(&mut self, x: i32, y: i32) {
        let mut dir_x = None;
        let mut dir_y = None;
        if x > self.x() {
            dir_x = Some((Direction::Right, x - self.x()));
        } else if x < self.x() {
            dir_x = Some((Direction::Left, self.x() - x));
        }
        if y > self.y() {
            dir_y = Some((Direction::Front, y - self.y()));
        } else if y < self.y() {
            dir_y = Some((Direction::Back, self.y() - y));
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
                self.action = EnemyAction::None;
            }
        }
    }

    pub fn update(&mut self, player: &Player, map: &Map) {
        let distance = utils::compute_distance(player, self);
        match self.action {
            EnemyAction::None | EnemyAction::MoveTo(..)
                if distance < (::std::cmp::min(self.height(), self.width()) * 2) as i32 =>
            {
                println!("Enemy is gonna chase player!");
                self.action = EnemyAction::MoveToPlayer;
            }
            EnemyAction::None => {
                let mut x = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
                let mut y = rand::thread_rng().gen::<i32>() % MAX_DISTANCE_WANDERING;
                if x > -20 && x < 20 && y > -20 && y < 20 {
                    x = 20 * if x < 0 { -1 } else { 1 };
                    y = 20 * if y < 0 { -1 } else { 1 };
                }
                self.action = EnemyAction::MoveTo(x + self.start_x, y + self.start_y);
                println!(
                    "Enemy is gonna move to ({} {})",
                    x + self.start_x,
                    y + self.start_y
                );
            }
            EnemyAction::MoveTo(x, y) => {
                if utils::compute_distance(self, &(x, y)) < 20 {
                    println!("Enemy reached destination!");
                    // We reached the goal, let's find another one. :)
                    self.action = EnemyAction::None;
                    self.character.action.movement = None;
                } else {
                    self.compute_destination(x, y);
                    if !self.character.inner_apply_move(map) {
                        println!("Enemy cannot move forward");
                        self.action = EnemyAction::None;
                        self.character.action.movement = None;
                    } else {
                        self.character.action.movement = Some(0);
                        println!("Enemy is moving");
                    }
                }
            }
            EnemyAction::MoveToPlayer => {
                if distance > MAX_DISTANCE_PURSUIT {
                    println!("Enemy stop chasing player (player too far)");
                    // We come back to the initial position
                    self.action = EnemyAction::MoveTo(self.start_x, self.start_y);
                    self.character.action.movement = None;
                } else if distance < player.width() as i32 || distance < player.height() as i32 {
                    println!("Enemy stop chasing player (reached player)");
                    self.action = EnemyAction::None;
                    self.character.action.movement = None;
                } else {
                    println!("Enemy chasing player");
                    self.compute_destination(player.x(), player.y());
                    self.character.action.movement = Some(0);
                    self.character.inner_apply_move(map);
                }
            }
        }
    }
}

impl<'a> GetPos for Enemy<'a> {
    fn x(&self) -> i32 {
        self.character.x
    }

    fn y(&self) -> i32 {
        self.character.y
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
