use rand::Rng;
use rand_chacha::ChaCha8Rng;
use sdl2::image::LoadSurface;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::MAP_SIZE;

fn draw_in_map(
    map: &mut [u8],
    surface_map: &mut Surface,
    surface: &Surface,
    rng: &mut ChaCha8Rng,
    value: u8,
) -> bool {
    let pos: u32 = rng.gen::<u32>() % (MAP_SIZE * MAP_SIZE - 1);
    let pos_x = pos % MAP_SIZE;
    let pos_y = pos / MAP_SIZE;

    // First we check there is nothing there...
    for y in 0..surface.height() / 8 {
        for x in 0..surface.width() / 8 {
            let i = pos_x + x + (y + pos_y) * MAP_SIZE;
            if i < MAP_SIZE * MAP_SIZE && map[i as usize] != 0 {
                return false;
            }
        }
    }

    for y in 0..surface.height() / 8 {
        for x in 0..surface.width() / 8 {
            let i = pos_x + x + (y + pos_y) * MAP_SIZE;
            if i < MAP_SIZE * MAP_SIZE {
                map[i as usize] = value;
            }
        }
    }
    surface.blit(
        None,
        surface_map,
        Rect::new(
            pos_x as i32 * 8,
            pos_y as i32 * 8,
            surface.width(),
            surface.height(),
        ),
    ).expect("failed to blit");
    true
}

pub struct Map<'a> {
    pub data: Vec<u8>,
    pub x: i32,
    pub y: i32,
    pub texture: Texture<'a>,
}

impl<'a> Map<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, rng: &mut ChaCha8Rng) -> Map<'a> {
        let tree =
            Surface::from_file("resources/tree.png").expect("failed to load `resources/tree.png`");
        let bush =
            Surface::from_file("resources/bush.png").expect("failed to load `resources/bush.png`");
        let mut surface_map = Surface::new(
            MAP_SIZE * 8,
            MAP_SIZE * 8,
            texture_creator.default_pixel_format(),
        )
        .expect("failed to create map surface");
        surface_map
            .fill_rect(None, Color::RGB(80, 216, 72))
            .expect("failed to fill surface map");

        let mut map = vec![0; (MAP_SIZE * MAP_SIZE) as usize];

        // We first create trees
        for _ in 0..200 {
            loop {
                if draw_in_map(&mut map, &mut surface_map, &tree, rng, 1) {
                    break;
                }
            }
        }
        // We then create bushes
        for _ in 0..500 {
            loop {
                if draw_in_map(&mut map, &mut surface_map, &bush, rng, 2) {
                    break;
                }
            }
        }

        Map {
            data: map,
            x: MAP_SIZE as i32 * 8 / -2,
            y: MAP_SIZE as i32 * 8 / -2,
            texture: texture_creator
                .create_texture_from_surface(surface_map)
                .expect("failed to build texture from surface"),
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, screen: &Rect) {
        let x = screen.x - self.x;
        let y = screen.y - self.y;
        let (s_x, pos_x, width) = if x < 0 {
            (0, x * -1, (screen.width() as i32 + x) as u32)
        } else if x + screen.width() as i32 > MAP_SIZE as i32 * 8 {
            let sub = screen.width() as i32 - (screen.width() as i32 + x - MAP_SIZE as i32 * 8);
            (x, 0, sub as u32)
        } else {
            (x, 0, screen.width() as u32)
        };
        let (s_y, pos_y, height) = if y < 0 {
            (0, y * -1, (screen.height() as i32 + y) as u32)
        } else if y + screen.height() as i32 > MAP_SIZE as i32 * 8 {
            let sub = screen.height() as i32 - (screen.height() as i32 + y - MAP_SIZE as i32 * 8);
            (y, 0, sub as u32)
        } else {
            (y, 0, screen.height())
        };
        canvas
            .copy(
                &self.texture,
                Rect::new(s_x, s_y, width, height),
                Rect::new(pos_x, pos_y, width, height),
            )
            .expect("copy map failed");
    }
}
