use rand::Rng;
use rand_chacha::ChaCha8Rng;
use sdl2::image::LoadSurface;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::system::System;
use crate::{MAP_CASE_SIZE, MAP_SIZE};

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
    for y in 0..surface.height() / MAP_CASE_SIZE as u32 {
        for x in 0..surface.width() / MAP_CASE_SIZE as u32 {
            let i = pos_x + x + (y + pos_y) * MAP_SIZE;
            if i < MAP_SIZE * MAP_SIZE && map[i as usize] != 0 {
                return false;
            }
        }
    }

    for y in 0..surface.height() / MAP_CASE_SIZE as u32 {
        for x in 0..surface.width() / MAP_CASE_SIZE as u32 {
            let i = pos_x + x + (y + pos_y) * MAP_SIZE;
            if i < MAP_SIZE * MAP_SIZE {
                map[i as usize] = value;
            }
        }
    }
    surface
        .blit(
            None,
            surface_map,
            Rect::new(
                pos_x as i32 * MAP_CASE_SIZE,
                pos_y as i32 * MAP_CASE_SIZE,
                surface.width(),
                surface.height(),
            ),
        )
        .expect("failed to blit");
    true
}

pub struct Map<'a> {
    pub data: Vec<u8>,
    pub x: i32,
    pub y: i32,
    pub texture: Texture<'a>,
}

impl<'a> Map<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        rng: &mut ChaCha8Rng,
        x: i32,
        y: i32,
    ) -> Map<'a> {
        let tree =
            Surface::from_file("resources/tree.png").expect("failed to load `resources/tree.png`");
        let bush =
            Surface::from_file("resources/bush.png").expect("failed to load `resources/bush.png`");
        let mut surface_map = Surface::new(
            MAP_SIZE * MAP_CASE_SIZE as u32,
            MAP_SIZE * MAP_CASE_SIZE as u32,
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
            x,
            y,
            texture: texture_creator
                .create_texture_from_surface(surface_map)
                .expect("failed to build texture from surface"),
        }
    }

    pub fn draw(&self, system: &mut System) {
        let x = system.x() - self.x;
        let y = system.y() - self.y;
        let (s_x, pos_x, width) = if x < 0 {
            (0, x * -1, (system.width() + x) as u32)
        } else if x + system.width() > MAP_SIZE as i32 * MAP_CASE_SIZE {
            let sub = system.width() - (system.width() + x - MAP_SIZE as i32 * MAP_CASE_SIZE);
            (x, 0, sub as u32)
        } else {
            (x, 0, system.width() as u32)
        };
        let (s_y, pos_y, height) = if y < 0 {
            (0, y * -1, (system.height() + y) as u32)
        } else if y + system.height() > MAP_SIZE as i32 * MAP_CASE_SIZE {
            let sub = system.height() - (system.height() + y - MAP_SIZE as i32 * MAP_CASE_SIZE);
            (y, 0, sub as u32)
        } else {
            (y, 0, system.height() as u32)
        };
        system
            .canvas
            .copy(
                &self.texture,
                Rect::new(s_x, s_y, width, height),
                Rect::new(pos_x, pos_y, width, height),
            )
            .expect("copy map failed");
    }
}
