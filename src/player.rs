use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use sdl2::image::LoadSurface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::character::{Action, Character, CharacterKind, Direction};
use crate::player_stats::PlayerStats;
use crate::stat::Stat;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::weapon::Sword;
use crate::{GetDimension, GetPos, Id, ONE_SECOND};

pub fn create_right_actions<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    actions_standing: &[Dimension],
) -> (Texture<'a>, Surface<'a>) {
    let mut surface =
        Surface::from_file("resources/zelda.png").expect("failed to load `resources/zelda.png`");

    if surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
        surface = surface
            .convert_format(PixelFormatEnum::RGBA8888)
            .expect("failed to convert surface to RGBA8888");
    }

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
    pub stats: Option<RefCell<PlayerStats>>,
}

impl<'a> Player<'a> {
    pub const tile_width: u32 = 23;
    pub const tile_height: u32 = 23;

    pub fn get_actions_standing() -> Vec<Dimension> {
        vec![
            Dimension::new(Rect::new(15, 9, Self::tile_width, Self::tile_height), 0),
            Dimension::new(Rect::new(78, 9, Self::tile_width, Self::tile_height), 0),
            Dimension::new(Rect::new(51, 9, Self::tile_width, Self::tile_height), 0),
            Dimension::new(Rect::new(100, 9, Self::tile_width, Self::tile_height), 0),
        ]
    }

    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        texture: &'a Texture<'a>,
        surface: &'a Surface<'a>,
        x: i64,
        y: i64,
        id: Id,
        stats: Option<PlayerStats>,
    ) -> Player<'a> {
        let mut actions_moving = Vec::with_capacity(4);
        actions_moving.push((
            Dimension::new(Rect::new(15, 77, Self::tile_width, Self::tile_height), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(683, 77, Self::tile_width, Self::tile_height), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(350, 77, Self::tile_width, Self::tile_height), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(346, 44, Self::tile_width, Self::tile_height), 32),
            10,
        ));
        let texture_handler = TextureHandler::new(
            surface,
            texture,
            Self::get_actions_standing(),
            actions_moving,
            None,
        );

        Player {
            character: Character {
                action: Action {
                    direction: Direction::Up,
                    secondary: None,
                    movement: None,
                },
                x,
                y,
                health: Stat::new(1., 100000),
                mana: Stat::new(1., 100),
                stamina: Stat::new(30., 200),
                xp_to_next_level: 1000,
                xp: 990,
                texture_handler,
                weapon: Some(Sword::new(texture_creator, 10)),
                is_running: false,
                id,
                invincible_against: Vec::new(),
                statuses: Vec::new(),
                speed: ONE_SECOND / 60, // we want to move 60 times per second
                move_delay: 0,
                // TODO: take care if there are multiple local players: depending on where we want
                // to put the second player information, we might want to set this to "true".
                show_health_bar: false,
                death_animation: None,
                kind: CharacterKind::Player,
                effect: RefCell::new(None),
                level: 1,
                animations: Vec::new(),
            },
            is_run_pressed: false,
            stats: stats.map(|s| RefCell::new(s)),
        }
    }

    pub fn handle_move(&mut self, dir: Direction) {
        if self.character.action.movement.is_none() {
            if self.character.action.direction != dir {
                self.character.stop_attack();
            }
            self.character.action.direction = dir;
            self.character.action.movement = Some(0);
            self.character.is_running = self.is_run_pressed && self.character.stamina.value() > 0;
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
    fn x(&self) -> i64 {
        self.character.x
    }

    fn y(&self) -> i64 {
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
