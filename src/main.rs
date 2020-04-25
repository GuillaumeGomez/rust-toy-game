extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::image;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::time::Duration;

const TILE_WIDTH: u32 = 23;
const TILE_HEIGHT: u32 = 23;
const STANDING_FRONT: (i32, i32) = (15, 9);
const STANDING_LEFT: (i32, i32) = (51, 9);
const STANDING_RIGHT: (i32, i32) = (100, 9);
const STANDING_BACK: (i32, i32) = (78, 9);
const FRONT_MOVE: (i32, i32) = (15, 77);
const LEFT_MOVE: (i32, i32) = (350, 77);
const RIGHT_MOVE: (i32, i32) = (350, 50);
const BACK_MOVE: (i32, i32) = (683, 77);
const INCR: i32 = 32;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Front,
    Left,
    Right,
    Back,
}

struct Action {
    direction: Direction,
    movement: Option<u64>,
}

impl Action {
    fn get_current(&self) -> (i32, i32) {
        if let Some(ref pos) = self.movement {
            let pos = (pos % 60) as i32 / 6;
            match self.direction {
                Direction::Front => {
                    let (x, y) = FRONT_MOVE;
                    (pos * INCR + x, y)
                }
                Direction::Left => {
                    let (x, y) = LEFT_MOVE;
                    (pos * INCR + x, y)
                }
                Direction::Right => {
                    let (x, y) = RIGHT_MOVE;
                    (pos * INCR + x, y)
                }
                Direction::Back => {
                    let (x, y) = BACK_MOVE;
                    (pos * INCR + x, y)
                }
            }
        } else {
            match self.direction {
                Direction::Front => STANDING_FRONT,
                Direction::Left => STANDING_LEFT,
                Direction::Right => STANDING_RIGHT,
                Direction::Back => STANDING_BACK,
            }
        }
    }
}

fn create_right_actions<'a>(texture_creator: &'a TextureCreator<WindowContext>) -> Texture<'a> {
    let mut surface = Surface::from_file("resources/zelda.png")
        .expect("failed to load `resources/zelda.png`");

    let width = surface.width();
    let block_size = surface.pitch() / width;

    surface.with_lock_mut(|data| {
        let (src_x, src_y) = STANDING_LEFT;
        let (dest_x, dest_y) = STANDING_RIGHT;

        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                for tmp in 0..block_size {
                    let dest = tmp + (TILE_WIDTH - x + dest_x as u32 - 6) * block_size + (y + dest_y as u32) * width * block_size;
                    let src = tmp + (x + src_x as u32) * block_size + (y + src_y as u32) * width * block_size;
                    data[dest as usize] = data[src as usize];
                }
            }
        }
        let (src_x, src_y) = LEFT_MOVE;
        let (dest_x, dest_y) = RIGHT_MOVE;
        let max = 10 * INCR as u32 - (INCR as u32 - TILE_WIDTH);

        for y in 0..TILE_HEIGHT {
            for x in 0..max {
                for tmp in 0..block_size {
                    let dest = tmp + (max - x + dest_x as u32 - 4) * block_size + (y + dest_y as u32) * width * block_size;
                    let src = tmp + (x + src_x as u32) * block_size + (y + src_y as u32) * width * block_size;
                    data[dest as usize] = data[src as usize];
                }
            }
        }
    });

    texture_creator.create_texture_from_surface(surface).expect("failed to build texture from surface")
}

macro_rules! handle_move {
    ($current:ident, $dir:path) => (
        if $current.movement.is_none() {
            $current.direction = $dir;
            $current.movement = Some(0);
        }
    )
}

pub fn main() {
    let sdl_context = sdl2::init().expect("failed to init SDL");
    let _sdl_img_context = image::init(image::InitFlag::PNG).expect("failed to init SDL image");
    let video_subsystem = sdl_context.video().expect("failed to get video context");

    let window = video_subsystem.window("sdl2 demo", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .expect("failed to build window");

    let mut canvas: Canvas<Window> = window.into_canvas()
        .present_vsync()
        .build()
        .expect("failed to build window's canvas");
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let texture_creator = canvas.texture_creator();
    let texture = create_right_actions(&texture_creator);

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let mut current = Action {
        direction: Direction::Front,
        movement: None,
    };

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                Event::KeyDown { keycode: Some(x), .. } => {
                    match x {
                        Keycode::Escape => break 'running,
                        Keycode::Left => handle_move!(current, Direction::Left),
                        Keycode::Right => handle_move!(current, Direction::Right),
                        Keycode::Up => handle_move!(current, Direction::Back),
                        Keycode::Down => handle_move!(current, Direction::Front),
                        _ => {}
                    }
                }
                Event::KeyUp { keycode: Some(x), .. } => {
                    match x {
                        // Not complete: if a second direction is pressed, it should then go to this
                        // direction. :)
                        Keycode::Left => current.movement = None,
                        Keycode::Right => current.movement = None,
                        Keycode::Up => current.movement = None,
                        Keycode::Down => current.movement = None,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        canvas.present();
        canvas.clear();

        let width = WIDTH / 2 - TILE_WIDTH / 2;
        let height = HEIGHT / 2 - TILE_HEIGHT / 2;
        let (x, y) = current.get_current();
        canvas.copy(
            &texture,
            Rect::new(x, y, TILE_WIDTH, TILE_HEIGHT),
            Rect::new(width as _, height as _, TILE_WIDTH, TILE_HEIGHT),
        ).expect("copy failed");
        if let Some(ref mut pos) = current.movement {
            *pos += 1;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
