use std::cell::RefCell;
use std::cmp::Ordering;

use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::PixelFormatEnum;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::TextureCreator;
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

use crate::animation::Animation;
use crate::character::{Action, Character, CharacterKind, CharacterPoints, Direction, Obstacle};
use crate::enemy::{Enemy, EnemyAction};
use crate::map::Map;
use crate::player::Player;
use crate::texture_handler::{Dimension, TextureHandler};
use crate::texture_holder::{TextureHolder, TextureId, Textures};
use crate::utils;
use crate::weapons::Nothing;
use crate::{
    GetDimension, GetPos, Id, MAP_CASE_SIZE, MAX_DISTANCE_PURSUIT, MAX_DISTANCE_WANDERING,
    ONE_SECOND,
};

pub struct Bat {
    pub character: Character,
    action: RefCell<EnemyAction>,
    start_x: f32,
    start_y: f32,
}

impl Bat {
    pub fn init_textures<'a>(
        texture_creator: &'a TextureCreator<WindowContext>,
        textures: &mut Textures<'a>,
    ) {
        let mut surface =
            Surface::from_file("resources/bat.png").expect("failed to load `resources/bat.png`");

        if surface.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
            surface = surface
                .convert_format(PixelFormatEnum::RGBA8888)
                .expect("failed to convert surface to RGBA8888");
        }

        textures.add_named_texture(
            "bat",
            TextureHolder::surface_to_texture(texture_creator, &surface),
        );
        textures.add_surface("bat", surface);
    }

    pub fn new<'a>(
        texture_creator: &TextureCreator<WindowContext>,
        textures: &Textures<'a>,
        x: f32,
        y: f32,
        id: Id,
    ) -> Self {
        let tile_height = 13;
        let tile_width = 16;

        let actions_standing = vec![
            Dimension::new(Rect::new(tile_width as i32, 0, tile_width, tile_height), 0),
            Dimension::new(Rect::new(tile_width as i32, 0, tile_width, tile_height), 0),
            Dimension::new(Rect::new(tile_width as i32, 0, tile_width, tile_height), 0),
            Dimension::new(Rect::new(tile_width as i32, 0, tile_width, tile_height), 0),
        ];
        let actions_moving = vec![
            (
                Dimension::new(Rect::new(0, 0, tile_width, tile_height), tile_width as i32),
                5,
            ),
            (
                Dimension::new(Rect::new(0, 0, tile_width, tile_height), tile_width as i32),
                5,
            ),
            (
                Dimension::new(Rect::new(0, 0, tile_width, tile_height), tile_width as i32),
                5,
            ),
            (
                Dimension::new(Rect::new(0, 0, tile_width, tile_height), tile_width as i32),
                5,
            ),
        ];

        // let texture = create_right_actions(&texture_creator, &actions_standing, &actions_moving);
        let texture_handler = TextureHandler::new(
            "bat",
            textures.get_texture_id_from_name("bat"),
            vec![],
            actions_moving,
            None,
        );

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
                    direction: Direction::Down,
                    secondary: None,
                    movement: Some(0),
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
                weapon: Nothing::new(0),
                is_running: false,
                id,
                invincible_against: Vec::new(),
                statuses: Vec::new(),
                speed: ONE_SECOND / 45,
                move_delay: 0,
                tile_duration: ONE_SECOND / 6,
                tile_delay: 0,
                show_health_bar: true,
                death_animation: Some(Animation::new_death(textures)),
                kind: CharacterKind::Enemy,
                effect: RefCell::new(None),
                animations: Vec::new(),
                move_hitbox: (tile_width, tile_height),
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

    fn get_directions(&self, x_add: f32, y_add: f32) -> (Direction, Option<Direction>) {
        (Direction::Down, None)
    }
}

impl Enemy for Bat {
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
        // if !self.character.is_attacking() && self.action.borrow().is_attack() {
        //     self.character.attack();
        //     if x == 0 && y == 0 {
        //         match &*self.action.borrow() {
        //             EnemyAction::Attack(ref dir) => self.character.action.direction = *dir,
        //             _ => {}
        //         }
        //     }
        // }
        self.character.update(elapsed, x, y, None)
    }

    fn apply_move(
        &self,
        map: &Map,
        _elapsed: u32,
        players: &[Player],
        npcs: &[Box<dyn Enemy>],
    ) -> (f32, f32) {
        (0., 0.)
    }

    fn draw(&mut self, system: &mut crate::system::System, debug: bool) {
        // use crate::sdl2::rect::Point;
        // if debug {
        //     match &*self.action.borrow() {
        //         EnemyAction::MoveTo(ref nodes) | EnemyAction::MoveToPlayer(ref nodes) => {
        //             let mut iter = nodes.iter().peekable();
        //             while let Some(node) = iter.next() {
        //                 if let Some(next) = iter.peek() {
        //                     system
        //                         .canvas
        //                         .draw_line(
        //                             Point::new(
        //                                 (next.0 - system.x()) as i32,
        //                                 (next.1 - system.y()) as i32,
        //                             ),
        //                             Point::new(
        //                                 (node.0 - system.x()) as i32,
        //                                 (node.1 - system.y()) as i32,
        //                             ),
        //                         )
        //                         .unwrap();
        //                 } else {
        //                     system
        //                         .canvas
        //                         .draw_line(
        //                             Point::new(
        //                                 (self.x() - system.x()) as i32,
        //                                 (self.y() - system.y()) as i32,
        //                             ),
        //                             Point::new(
        //                                 (node.0 - system.x()) as i32,
        //                                 (node.1 - system.y()) as i32,
        //                             ),
        //                         )
        //                         .unwrap();
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        self.character.draw(system, debug);
    }
}
