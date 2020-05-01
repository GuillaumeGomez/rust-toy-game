use sdl2::image::LoadSurface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::{Action, Direction};
use crate::character::Character;
use crate::map::Map;
use crate::texture_handler::{Dimension, TextureHandler};

pub struct Enemy<'a> {
    character: Character<'a>,
}

impl<'a> Enemy<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, x: i32, y: i32) -> Enemy<'a> {
        let mut actions_standing = Vec::with_capacity(4);

        // front
        actions_standing.push(
            Dimension::new(Rect::new(0, 73, 28, 36), 0),
        );
        // left
        actions_standing.push(
            Dimension::new(Rect::new(0, 40, 44, 31), 0),
        );
        // right
        actions_standing.push(
            Dimension::new(Rect::new(0, 40, 44, 31), 0),
        );
        // back
        actions_standing.push(
            Dimension::new(Rect::new(0, 29, 29, 52), 0),
        );
        let mut actions_moving = Vec::with_capacity(4);
        let tile_width = 1;
        let tile_height = 1;
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

        let surface =
            Surface::from_file("resources/enemy1.png").expect("failed to load `resources/enemy1.png`");

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
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        self.character.draw(canvas, false, screen)
    }

    pub fn apply_move(&mut self, map: &Map) {
        // todo
    }
}
