use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::PixelFormatEnum;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::character::{Action, Character, CharacterKind, CharacterPoints, Direction};
use crate::env::Env;
use crate::player_stats::PlayerStats;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::texture_holder::{TextureHolder, Textures};
use crate::weapons::Sword;
use crate::{GetDimension, GetPos, Id, ONE_SECOND};

pub struct Player {
    pub character: Character,
    pub is_run_pressed: bool,
    pub stats: Option<RefCell<PlayerStats>>,
}

const MARGIN_STANDING: u32 = 4;

impl Player {
    pub const TILE_WIDTH: u32 = 22;
    pub const TILE_HEIGHT: u32 = 22;

    pub fn init_textures<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
    ) {
        let mut surface = Surface::from_file("resources/zelda.png")
            .expect("failed to load `resources/zelda.png`");

        if surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
            surface = surface
                .convert_format(PixelFormatEnum::RGBA8888)
                .expect("failed to convert surface to RGBA8888");
        }

        textures.add_named_texture(
            "player",
            TextureHolder::surface_to_texture(texture_creator, &surface),
        );
        textures.add_surface("player", surface);
    }

    pub fn get_actions_standing() -> Vec<Dimension> {
        vec![
            Dimension::new(
                Rect::new(0, 0, Self::TILE_WIDTH - MARGIN_STANDING, Self::TILE_HEIGHT),
                0,
            ),
            Dimension::new(
                Rect::new(18, 0, Self::TILE_WIDTH - MARGIN_STANDING, Self::TILE_HEIGHT),
                0,
            ),
            Dimension::new(
                Rect::new(36, 0, Self::TILE_WIDTH - MARGIN_STANDING, Self::TILE_HEIGHT),
                0,
            ),
            Dimension::new(
                Rect::new(54, 0, Self::TILE_WIDTH - MARGIN_STANDING, Self::TILE_HEIGHT),
                0,
            ),
        ]
    }

    pub fn new<'a>(
        textures: &Textures<'a>,
        x: i64,
        y: i64,
        id: Id,
        stats: Option<PlayerStats>,
        _env: Option<&mut Env>,
    ) -> Self {
        let mut actions_moving = Vec::with_capacity(4);
        actions_moving.push((
            Dimension::new(Rect::new(15, 77, Self::TILE_WIDTH, Self::TILE_HEIGHT), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(683, 77, Self::TILE_WIDTH, Self::TILE_HEIGHT), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(350, 77, Self::TILE_WIDTH, Self::TILE_HEIGHT), 32),
            10,
        ));
        actions_moving.push((
            Dimension::new(Rect::new(346, 44, Self::TILE_WIDTH, Self::TILE_HEIGHT), 32),
            10,
        ));
        let texture_handler = TextureHandler::new(
            "player",
            textures.get_texture_id_from_name("player"),
            Self::get_actions_standing(),
            actions_moving,
            None,
        );

        // health: Stat::new(1., 100000),
        // mana: Stat::new(1., 100),
        // stamina: Stat::new(30., 200),
        let level = 1;
        let points = CharacterPoints {
            strength: 1,
            constitution: 100,
            intelligence: 1,
            wisdom: 1,
            stamina: 10,
            agility: 10,
            dexterity: 1,
        };
        let p_stats = points.generate_stats(level);
        let p = Player {
            character: Character {
                action: Action {
                    direction: Direction::Up,
                    secondary: None,
                    movement: None,
                },
                x,
                y,
                points,
                stats: p_stats,
                xp_to_next_level: 1000,
                xp: 990,
                level,
                unused_points: 0,
                texture_handler,
                weapon: Sword::new(textures, 10),
                is_running: false,
                id,
                invincible_against: Vec::new(),
                statuses: Vec::new(),
                speed: ONE_SECOND / 60, // we want to move 60 times per second
                move_delay: 0,
                tile_duration: ONE_SECOND / 8,
                tile_delay: 0,
                // TODO: take care if there are multiple local players: depending on where we want
                // to put the second player information, we might want to set this to "true".
                show_health_bar: false,
                death_animation: None,
                kind: CharacterKind::Player,
                effect: RefCell::new(None),
                animations: Vec::new(),
                move_hitbox: (Self::TILE_WIDTH - MARGIN_STANDING, 6),
                blocking_direction: None,
                weapon_action: None,
            },
            is_run_pressed: false,
            stats: stats.map(|s| RefCell::new(s)),
        };
        // if let Some(env) = env {
        //     env.add_character_update("Level", UpdateKind::Value(p.level as _));
        //     env.add_character_update("Experience", UpdateKind::Both(p.xp, p.xp_to_next_level));
        //     env.add_character_update(
        //         "Stamina",
        //         UpdateKind::Both(p.stamina.value(), p.stamina.max_value()),
        //     );
        //     env.add_character_update(
        //         "Health",
        //         UpdateKind::Both(p.health.value(), p.health.max_value()),
        //     );
        //     env.add_character_update("Mana", UpdateKind::Both(p.mana.value(), p.mana.max_value()));
        // }
        p
    }

    pub fn handle_move(&mut self, dir: Direction) {
        if self.character.action.movement.is_none() {
            if self.character.action.direction != dir {
                self.character.stop_attack();
            }
            self.character.action.direction = dir;
            self.character.action.movement = Some(0);
            self.character.is_running =
                self.is_run_pressed && self.character.stats.stamina.value() > 0;
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

impl GetPos for Player {
    fn x(&self) -> i64 {
        self.character.x
    }

    fn y(&self) -> i64 {
        self.character.y
    }
}

impl GetDimension for Player {
    fn width(&self) -> u32 {
        self.character.width()
    }
    fn height(&self) -> u32 {
        self.character.height()
    }
}

impl Deref for Player {
    type Target = Character;

    fn deref(&self) -> &Self::Target {
        &self.character
    }
}

impl DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.character
    }
}
