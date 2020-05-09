use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::character::{Action, Character, Direction};
use crate::map::Map;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::weapon::Sword;
use crate::{GetDimension, GetPos, Id};

fn create_right_actions<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    actions_standing: &[Dimension],
    _actions_moving: &[(Dimension, i32)],
) -> (Texture<'a>, Surface<'a>) {
    let mut surface =
        Surface::from_file("resources/zelda.png").expect("failed to load `resources/zelda.png`");

    let width = surface.width();
    let block_size = surface.pitch() / width;

    surface.with_lock_mut(|data| {
        let left = &actions_standing[Direction::Left as usize];
        let (src_x, src_y) = (left.x, left.y);
        let right = &actions_standing[Direction::Right as usize];
        let (dest_x, dest_y) = (right.x, right.y);

        for y in 0..left.height() {
            for x in 0..left.width() {
                for tmp in 0..block_size {
                    let dest = tmp
                        + (left.width() - x + dest_x as u32 - 6) * block_size
                        + (y + dest_y as u32) * width * block_size;
                    let src = tmp
                        + (x + src_x as u32) * block_size
                        + (y + src_y as u32) * width * block_size;
                    data[dest as usize] = data[src as usize];
                }
            }
        }
    });

    (
        texture_creator
            .create_texture_from_surface(&surface)
            .expect("failed to build texture from surface"),
        surface,
    )
}

pub struct Player<'a> {
    pub character: Character<'a>,
    pub is_run_pressed: bool,
}

impl<'a> Player<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        x: i32,
        y: i32,
        id: Id,
    ) -> Player<'a> {
        let tile_width = 23;
        let tile_height = 23;
        let mut actions_standing = Vec::with_capacity(4);
        actions_standing.push(Dimension::new(Rect::new(15, 9, tile_width, tile_height), 0));
        actions_standing.push(Dimension::new(Rect::new(78, 9, tile_width, tile_height), 0));
        actions_standing.push(Dimension::new(Rect::new(51, 9, tile_width, tile_height), 0));
        actions_standing.push(Dimension::new(
            Rect::new(100, 9, tile_width, tile_height),
            0,
        ));
        let mut actions_moving = Vec::with_capacity(4);
        actions_moving.push((
            Dimension::new(Rect::new(15, 77, tile_width, tile_height), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(683, 77, tile_width, tile_height), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(350, 77, tile_width, tile_height), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(346, 44, tile_width, tile_height), 32),
            10,
        ));
        let (texture, surface) =
            create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler =
            TextureHandler::new(surface, texture, actions_standing, actions_moving);

        Player {
            character: Character {
                action: Action {
                    direction: Direction::Up,
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
                weapon: Some(Sword::new(texture_creator)),
                is_running: false,
                id,
                invincible_against: HashMap::new(),
            },
            is_run_pressed: false,
        }
    }

    pub fn handle_move(&mut self, dir: Direction) {
        if self.character.action.movement.is_none() {
            if self.character.action.direction != dir {
                self.character.stop_attack();
            }
            self.character.action.direction = dir;
            self.character.action.movement = Some(0);
            self.character.is_running = self.is_run_pressed && self.character.stamina > 0;
        } else if self.character.action.secondary.is_none()
            && dir != self.character.action.direction
        {
            self.character.action.secondary = Some(dir);
        }
    }

    pub fn handle_release(&mut self, dir: Direction) {
        if Some(dir) == self.character.action.secondary {
            self.character.action.secondary = None;
        } else if dir == self.character.action.direction {
            if let Some(second) = self.character.action.secondary.take() {
                self.character.action.movement = Some(0);
                self.character.action.direction = second;
                self.character.stop_attack();
            } else {
                self.character.action.movement = None;
                self.character.is_running = false;
            }
        }
    }
}

impl<'a> GetPos for Player<'a> {
    fn x(&self) -> i32 {
        self.character.x
    }

    fn y(&self) -> i32 {
        self.character.y
    }
}

impl<'a> GetDimension for Player<'a> {
    fn width(&self) -> u32 {
        self.character.width()
    }
    fn height(&self) -> u32 {
        self.character.height()
    }
}

impl<'a> Deref for Player<'a> {
    type Target = Character<'a>;

    fn deref(&self) -> &Self::Target {
        &self.character
    }
}

impl<'a> DerefMut for Player<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.character
    }
}
