use rand::Rng;
use rand_chacha::ChaCha8Rng;
use sdl2::image::LoadSurface;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::system::System;
use crate::{MAP_CASE_SIZE, MAP_SIZE};

fn check_pixels_for_pos(x: u32, y: u32, surface: &Surface) -> bool {
    let x = x as usize;
    let y = y as usize;
    let pitch = surface.pitch() as usize;
    let max_len = (surface.height() * surface.pitch()) as usize;
    let surface = surface.raw();
    let pixels = unsafe { (*surface).pixels as *const u8 };

    for iy in 0..MAP_CASE_SIZE as usize {
        let y = (iy + y) * pitch;
        for ix in 0..MAP_CASE_SIZE as usize {
            let pos = y + (ix + x) * 4; // 4 is because the surfaces are always RGBA8888 so 4 bytes
            if pos >= 0 && pos < max_len {
                let target_pixel = unsafe { *(pixels.add(pos as usize) as *const u32) };
                let alpha = target_pixel & 255;
                if alpha > 220 {
                    return true;
                }
            }
        }
    }
    false
}

fn draw_in_map(
    map: &mut [u8],
    surface_map: &mut Surface,
    surface_map_layer: &mut Surface,
    surface: &Surface,
    rng: &mut ChaCha8Rng,
    value: u8,
    real_size: Option<Rect>,
    top_layer: Option<Rect>,
) -> bool {
    let pos: u32 = rng.gen::<u32>() % (MAP_SIZE * MAP_SIZE - 1);
    let pos_x = pos % MAP_SIZE;
    let pos_y = pos / MAP_SIZE;

    let (x, y, width, height) = match real_size {
        Some(r) => (r.x, r.y, r.width(), r.height()),
        None => (0, 0, surface.width(), surface.height()),
    };

    // First we check there is nothing there...
    for y in 0..height / MAP_CASE_SIZE as u32 {
        let y = (y + pos_y) * MAP_SIZE;
        for x in 0..width / MAP_CASE_SIZE as u32 {
            let i = pos_x + x + y;
            if i < MAP_SIZE * MAP_SIZE && map[i as usize] != 0 {
                return false;
            }
        }
    }

    for iy in 0..height / MAP_CASE_SIZE as u32 {
        for ix in 0..width / MAP_CASE_SIZE as u32 {
            // First, we need to check if there is actually a pixel in the surface at this position.
            if check_pixels_for_pos(
                x as u32 + ix * MAP_CASE_SIZE as u32,
                y as u32 + iy * MAP_CASE_SIZE as u32,
                &surface,
            ) {
                if real_size.is_some() {
                    println!("toudoum! ({} {})", pos_x + x as u32, iy + pos_y);
                }
                let i = pos_x + x as u32 + (iy + pos_y) * MAP_SIZE;
                if i < MAP_SIZE * MAP_SIZE {
                    map[i as usize] = value;
                }
            }
        }
    }
    surface
        .blit(
            Rect::new(x, y, width, height),
            surface_map,
            Rect::new(
                pos_x as i32 * MAP_CASE_SIZE as i32,
                pos_y as i32 * MAP_CASE_SIZE as i32,
                width,
                height,
            ),
        )
        .expect("failed to blit");
    if let Some(top_layer) = top_layer {
        surface
            .blit(
                top_layer,
                surface_map_layer,
                Rect::new(
                    pos_x as i32 * MAP_CASE_SIZE as i32 - (x - top_layer.x),
                    pos_y as i32 * MAP_CASE_SIZE as i32 - top_layer.height() as i32,
                    top_layer.width(),
                    top_layer.height(),
                ),
            )
            .expect("failed to blit");
    }
    true
}

pub struct Map<'a> {
    pub data: Vec<u8>,
    pub x: i64,
    pub y: i64,
    pub texture: Texture<'a>,
    pub top_layer_texture: Texture<'a>,
}

impl<'a> Map<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        rng: &mut ChaCha8Rng,
        x: i64,
        y: i64,
    ) -> Map<'a> {
        let mut tree = Surface::from_file("resources/trees.png")
            .expect("failed to load `resources/trees.png`");
        if tree.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
            tree = tree
                .convert_format(PixelFormatEnum::RGBA8888)
                .expect("failed to convert tree to RGBA8888");
        }
        let mut bush =
            Surface::from_file("resources/bush.png").expect("failed to load `resources/bush.png`");
        if tree.pixel_format_enum() != PixelFormatEnum::RGBA8888 {
            bush = bush
                .convert_format(PixelFormatEnum::RGBA8888)
                .expect("failed to convert bush to RGBA8888");
        }
        let mut surface_map = Surface::new(
            MAP_SIZE * MAP_CASE_SIZE as u32,
            MAP_SIZE * MAP_CASE_SIZE as u32,
            PixelFormatEnum::RGBA8888, // We need to force the alpha channel!
        )
        .expect("failed to create map surface");
        surface_map
            .fill_rect(None, Color::RGB(80, 216, 72))
            .expect("failed to fill surface map");

        // This is the layer going "on top" of the rest.
        let mut surface_map_layer = Surface::new(
            MAP_SIZE * MAP_CASE_SIZE as u32,
            MAP_SIZE * MAP_CASE_SIZE as u32,
            PixelFormatEnum::RGBA8888, // We need to force the alpha channel!
        )
        .expect("failed to create map surface layer");

        let mut map = vec![0; (MAP_SIZE * MAP_SIZE) as usize];

        // We first create trees
        for _ in 0..200 {
            loop {
                if draw_in_map(
                    &mut map,
                    &mut surface_map,
                    &mut surface_map_layer,
                    &tree,
                    rng,
                    1,
                    Some(Rect::new(184, 99, 60, 27)),
                    Some(Rect::new(170, 0, 72, 99)),
                ) {
                    break;
                }
            }
        }
        // We then create bushes
        for _ in 0..500 {
            loop {
                if draw_in_map(
                    &mut map,
                    &mut surface_map,
                    &mut surface_map_layer,
                    &bush,
                    rng,
                    2,
                    None,
                    None,
                ) {
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
            top_layer_texture: texture_creator
                .create_texture_from_surface(surface_map_layer)
                .expect("failed to build texture from surface"),
        }
    }

    pub fn draw(&self, system: &mut System) {
        let x = system.x() - self.x;
        let y = system.y() - self.y;
        let (s_x, pos_x, width) = if x < 0 {
            (0, x * -1, (system.width() as i64 + x) as u32)
        } else if x + system.width() as i64 > MAP_SIZE as i64 * MAP_CASE_SIZE {
            let sub = system.width() as i64
                - (system.width() as i64 + x - MAP_SIZE as i64 * MAP_CASE_SIZE);
            (x, 0, sub as u32)
        } else {
            (x, 0, system.width() as u32)
        };
        let (s_y, pos_y, height) = if y < 0 {
            (0, y * -1, (system.height() as i64 + y) as u32)
        } else if y + system.height() as i64 > MAP_SIZE as i64 * MAP_CASE_SIZE {
            let sub = system.height() as i64
                - (system.height() as i64 + y - MAP_SIZE as i64 * MAP_CASE_SIZE);
            (y, 0, sub as u32)
        } else {
            (y, 0, system.height() as u32)
        };
        system
            .canvas
            .copy(
                &self.texture,
                Rect::new(s_x as i32, s_y as i32, width, height),
                Rect::new(pos_x as i32, pos_y as i32, width, height),
            )
            .expect("copy map failed");
    }

    pub fn draw_layer(&self, system: &mut System) {
        let x = system.x() - self.x;
        let y = system.y() - self.y;
        let (s_x, pos_x, width) = if x < 0 {
            (0, x * -1, (system.width() as i64 + x) as u32)
        } else if x + system.width() as i64 > MAP_SIZE as i64 * MAP_CASE_SIZE {
            let sub = system.width() as i64
                - (system.width() as i64 + x - MAP_SIZE as i64 * MAP_CASE_SIZE);
            (x, 0, sub as u32)
        } else {
            (x, 0, system.width() as u32)
        };
        let (s_y, pos_y, height) = if y < 0 {
            (0, y * -1, (system.height() as i64 + y) as u32)
        } else if y + system.height() as i64 > MAP_SIZE as i64 * MAP_CASE_SIZE {
            let sub = system.height() as i64
                - (system.height() as i64 + y - MAP_SIZE as i64 * MAP_CASE_SIZE);
            (y, 0, sub as u32)
        } else {
            (y, 0, system.height() as u32)
        };
        system
            .canvas
            .copy(
                &self.top_layer_texture,
                Rect::new(s_x as i32, s_y as i32, width, height),
                Rect::new(pos_x as i32, pos_y as i32, width, height),
            )
            .expect("copy map failed");
    }
}
