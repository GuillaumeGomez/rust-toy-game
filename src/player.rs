use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::{Action, Direction};
use crate::character::Character;
use crate::map::Map;
use crate::texture_handler::{Dimension, TextureHandler};

fn create_right_actions<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    actions_standing: &[Dimension],
    actions_moving: &[(Dimension, i32)],
) -> Texture<'a> {
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
        let (left, incr) = &actions_moving[Direction::Left as usize];
        let (src_x, src_y) = (left.x, left.y);
        let (right, _) = &actions_moving[Direction::Right as usize];
        let (dest_x, dest_y) = (right.x, right.y);
        let max = 10 * *incr - (*incr - left.width() as i32);
        let max = max as u32;

        for y in 0..left.height() {
            for x in 0..max {
                for tmp in 0..block_size {
                    let dest = tmp
                        + (max - x + dest_x as u32 - 4) * block_size
                        + (y + dest_y as u32) * width * block_size;
                    let src = tmp
                        + (x + src_x as u32) * block_size
                        + (y + src_y as u32) * width * block_size;
                    data[dest as usize] = data[src as usize];
                }
            }
        }
    });

    texture_creator
        .create_texture_from_surface(surface)
        .expect("failed to build texture from surface")
}

pub struct Player<'a> {
    pub character: Character<'a>,
    pub is_running: bool,
    pub is_run_pressed: bool,
}

impl<'a> Player<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, x: i32, y: i32) -> Player<'a> {
        let tile_width = 23;
        let tile_height = 23;
        let mut actions_standing = Vec::with_capacity(4);
        actions_standing.push(
            Dimension::new(Rect::new(15, 9, tile_width, tile_height), 0),
        );
        actions_standing.push(
            Dimension::new(Rect::new(51, 9, tile_width, tile_height), 0),
        );
        actions_standing.push(
            Dimension::new(Rect::new(100, 9, tile_width, tile_height), 0),
        );
        actions_standing.push(
            Dimension::new(Rect::new(78, 9, tile_width, tile_height), 0),
        );
        let mut actions_moving = Vec::with_capacity(4);
        actions_moving.push(
            (Dimension::new(Rect::new(15, 77, tile_width, tile_height), 32), 10),
        );
        actions_moving.push(
            (Dimension::new(Rect::new(350, 77, tile_width, tile_height), 32), 10),
        );
        actions_moving.push(
            (Dimension::new(Rect::new(350, 50, tile_width, tile_height), 32), 10),
        );
        actions_moving.push(
            (Dimension::new(Rect::new(683, 77, tile_width, tile_height), 32), 10),
        );
        let texture = create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler = TextureHandler {
            texture,
            actions_standing,
            actions_moving,
        };

        Player {
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
            is_running: false,
            is_run_pressed: false,
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        self.character.draw(canvas, self.is_running, screen)
    }

    pub fn apply_move(&mut self, map: &Map) {
        if self.character.inner_apply_move(map) {
            if self.is_running {
                self.character.inner_apply_move(map);
                if self.character.stamina > 0 {
                    self.character.stamina -= 1;
                    if self.character.stamina == 0 {
                        self.is_running = false;
                    }
                }
                return;
            }
        }
        if self.character.stamina < self.character.total_stamina {
            self.character.stamina += 1;
        }
    }

    pub fn handle_move(&mut self, dir: Direction) {
        if self.character.action.movement.is_none() {
            self.character.action.direction = dir;
            self.character.action.movement = Some(0);
            self.is_running = self.is_run_pressed && self.character.stamina > 0;
        } else if self.character.action.secondary.is_none() && dir != self.character.action.direction {
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
            } else {
                self.character.action.movement = None;
                self.is_running = false;
            }
        }
    }
}