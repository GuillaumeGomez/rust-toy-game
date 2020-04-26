extern crate sdl2;

use rand::{self, rngs::ThreadRng, Rng};
use sdl2::event::Event;
use sdl2::image;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
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
const MAP_SIZE: u32 = 1_000;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Front,
    Left,
    Right,
    Back,
}

struct Action {
    direction: Direction,
    secondary: Option<Direction>,
    movement: Option<u64>,
}

impl Action {
    fn get_current(&self, is_running: bool) -> (i32, i32) {
        if let Some(ref pos) = self.movement {
            let pos = if is_running {
                (pos % 30) as i32 / 3
            } else {
                (pos % 60) as i32 / 6
            };
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
    let mut surface =
        Surface::from_file("resources/zelda.png").expect("failed to load `resources/zelda.png`");

    let width = surface.width();
    let block_size = surface.pitch() / width;

    surface.with_lock_mut(|data| {
        let (src_x, src_y) = STANDING_LEFT;
        let (dest_x, dest_y) = STANDING_RIGHT;

        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                for tmp in 0..block_size {
                    let dest = tmp
                        + (TILE_WIDTH - x + dest_x as u32 - 6) * block_size
                        + (y + dest_y as u32) * width * block_size;
                    let src = tmp
                        + (x + src_x as u32) * block_size
                        + (y + src_y as u32) * width * block_size;
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

fn draw_in_map(
    map: &mut [u8],
    surface_map: &mut Surface,
    surface: &Surface,
    rng: &mut ThreadRng,
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

fn create_map<'a>(texture_creator: &'a TextureCreator<WindowContext>) -> Map<'a> {
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

    let mut rng = rand::thread_rng();

    // We first create trees
    for _ in 0..200 {
        loop {
            if draw_in_map(&mut map, &mut surface_map, &tree, &mut rng, 1) {
                break;
            }
        }
    }
    // We then create bushes
    for _ in 0..500 {
        loop {
            if draw_in_map(&mut map, &mut surface_map, &bush, &mut rng, 2) {
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

fn handle_move(player: &mut Character, dir: Direction) {
    if player.action.movement.is_none() {
        player.action.direction = dir;
        player.action.movement = Some(0);
        player.is_running = player.is_run_pressed && player.stamina > 0;
    } else if player.action.secondary.is_none() && dir != player.action.direction {
        player.action.secondary = Some(dir);
    }
}

fn handle_release(player: &mut Character, dir: Direction) {
    if Some(dir) == player.action.secondary {
        player.action.secondary = None;
    } else if dir == player.action.direction {
        if let Some(second) = player.action.secondary.take() {
            player.action.movement = Some(0);
            player.action.direction = second;
        } else {
            player.action.movement = None;
            player.is_running = false;
        }
    }
}

struct Map<'a> {
    data: Vec<u8>,
    x: i32,
    y: i32,
    texture: Texture<'a>,
}

struct Character {
    action: Action,
    x: i32,
    y: i32,
    total_health: u32,
    health: u32,
    total_mana: u32,
    mana: u32,
    total_stamina: u32,
    stamina: u32,
    xp_to_next_level: u32,
    xp: u32,
    is_running: bool,
    is_run_pressed: bool,
}

impl Character {
    fn move_result(&self, dir: Direction) -> ((i32, i32), (i32, i32)) {
        match dir {
            Direction::Front => ((0, 0), (TILE_HEIGHT as i32 / 2, 1)),
            Direction::Back => ((0, 0), (TILE_HEIGHT as i32 / -4, -1)),
            Direction::Left => ((TILE_WIDTH as i32 / -2, -1), (0, 0)),
            Direction::Right => ((TILE_WIDTH as i32 / 2, 1), (0, 0)),
        }
    }
    fn inner_apply_move(&mut self, map: &Map) -> bool {
        let ((mut x, mut x_add), (mut y, mut y_add)) = self.move_result(self.action.direction);
        if let Some(second) = self.action.secondary {
            let ((tmp_x, tmp_x_add), (tmp_y, tmp_y_add)) = self.move_result(second);
            x += tmp_x;
            x_add += tmp_x_add;
            y += tmp_y;
            y_add += tmp_y_add;
        }
        if self.y + y >= map.y + MAP_SIZE as i32 * 8
            || self.y + y < map.y
            || self.x + x >= map.x + MAP_SIZE as i32 * 8
            || self.x + x < map.x
        {
            return false;
        }
        let map_pos = (self.y + y - map.y) / 8 * MAP_SIZE as i32 + (self.x + x - map.x) / 8;
        println!(
            "{}|{} => ({}, {})",
            map.data.len(),
            map_pos,
            self.x + x,
            self.y + y
        );
        if map_pos < 0 || map_pos as usize >= map.data.len() {
            return false;
        } else if map.data[map_pos as usize] != 0 {
            println!("/!\\ {:?}", map.data[map_pos as usize]);
            return false;
        }
        self.x += x_add;
        self.y += y_add;
        true
    }
    fn apply_move(&mut self, map: &Map) {
        if let Some(ref mut pos) = self.action.movement {
            *pos += 1;
        } else {
            if self.stamina < self.total_stamina {
                self.stamina += 1;
            }
            return;
        }
        if !self.inner_apply_move(map) {
            return;
        }
        if self.is_running {
            self.inner_apply_move(map);
            if self.stamina > 0 {
                self.stamina -= 1;
                if self.stamina == 0 {
                    self.is_running = false;
                }
            }
        } else if self.stamina < self.total_stamina {
            self.stamina += 1;
        }
    }
}

#[inline]
fn create_bar<'a>(
    bar_name: &str,
    width: u32,
    height: u32,
    color: Color,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Texture<'a> {
    let mut bar = Surface::new(width, height, texture_creator.default_pixel_format())
        .expect(&format!("failed to create {} surface", bar_name));
    bar.fill_rect(None, color)
        .expect(&format!("failed to fill {} surface", bar_name));
    texture_creator
        .create_texture_from_surface(bar)
        .expect(&format!(
            "failed to build texture from {} surface",
            bar_name
        ))
}

struct HUD<'a> {
    bars: Texture<'a>,
    bars_width: u32,
    bars_height: u32,
    health_bar: Texture<'a>,
    mana_bar: Texture<'a>,
    stamina_bar: Texture<'a>,
    xp_bar: Texture<'a>,
}

impl<'a> HUD<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>) -> HUD<'a> {
        let bars =
            Surface::from_file("resources/bars.png").expect("failed to load `resources/bars.png`");
        let bars_width = bars.width();
        let bars_height = bars.height();
        let bars = texture_creator
            .create_texture_from_surface(bars)
            .expect("failed to build texture from bars surface");

        let health_bar = create_bar("health bar", 144, 4, Color::RGB(247, 0, 43), texture_creator);
        let mana_bar = create_bar("mana bar", 144, 4, Color::RGB(0, 153, 207), texture_creator);
        let stamina_bar = create_bar(
            "stamina bar",
            144,
            4,
            Color::RGB(149, 38, 172),
            texture_creator,
        );
        let xp_bar = create_bar("xp bar", 144, 2, Color::RGB(237, 170, 66), texture_creator);

        HUD {
            bars,
            bars_width,
            bars_height,
            health_bar,
            mana_bar,
            stamina_bar,
            xp_bar,
        }
    }

    fn draw(&self, player: &Character, canvas: &mut Canvas<Window>) {
        macro_rules! draw_bar {
            ($total:ident, $current:ident, $height:expr, $name:expr, $y:expr, $texture:ident) => {{
                let show = 144 * player.$current / player.$total;
                canvas
                    .copy(
                        &self.$texture,
                        Rect::new(0, 0, show, $height),
                        Rect::new(2, $y, show, $height),
                    )
                    .expect(concat!("copy ", $name, " bar failed"));
            }};
        }
        canvas
            .copy(
                &self.bars,
                None,
                Rect::new(0, 0, self.bars_width, self.bars_height),
            )
            .expect("copy bars failed");
        draw_bar!(total_health, health, 4, "health", 2, health_bar);
        draw_bar!(total_mana, mana, 4, "mana", 8, mana_bar);
        draw_bar!(total_stamina, stamina, 4, "stamina", 14, stamina_bar);
        draw_bar!(xp_to_next_level, xp, 2, "xp", 20, xp_bar);
    }
}

pub fn main() {
    let sdl_context = sdl2::init().expect("failed to init SDL");
    let _sdl_img_context = image::init(image::InitFlag::PNG).expect("failed to init SDL image");
    let video_subsystem = sdl_context.video().expect("failed to get video context");

    let window = video_subsystem
        .window("sdl2 demo", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .expect("failed to build window");

    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("failed to build window's canvas");
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let texture_creator = canvas.texture_creator();
    let texture = create_right_actions(&texture_creator);
    let map_data = create_map(&texture_creator);

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let mut player = Character {
        action: Action {
            direction: Direction::Front,
            secondary: None,
            movement: None,
        },
        x: 0,
        y: 0,
        total_health: 100,
        health: 75,
        total_mana: 100,
        mana: 20,
        total_stamina: 100,
        stamina: 100,
        xp_to_next_level: 1000,
        xp: 150,
        is_running: false,
        is_run_pressed: false,
    };
    let hud = HUD::new(&texture_creator);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(x), ..
                } => match x {
                    Keycode::Escape => break 'running,
                    Keycode::Left => handle_move(&mut player, Direction::Left),
                    Keycode::Right => handle_move(&mut player, Direction::Right),
                    Keycode::Up => handle_move(&mut player, Direction::Back),
                    Keycode::Down => handle_move(&mut player, Direction::Front),
                    Keycode::LShift => {
                        player.is_run_pressed = true;
                        player.is_running = player.action.movement.is_some();
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(x), ..
                } => {
                    match x {
                        // Not complete: if a second direction is pressed, it should then go to this
                        // direction. :)
                        Keycode::Left => handle_release(&mut player, Direction::Left),
                        Keycode::Right => handle_release(&mut player, Direction::Right),
                        Keycode::Up => handle_release(&mut player, Direction::Back),
                        Keycode::Down => handle_release(&mut player, Direction::Front),
                        Keycode::LShift => {
                            player.is_run_pressed = false;
                            player.is_running = false;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        canvas.present();
        canvas.clear();

        let x = player.x - map_data.x - WIDTH as i32 / 2;
        let y = player.y - map_data.y - HEIGHT as i32 / 2;
        let (s_x, pos_x, width) = if x < 0 {
            (0, x * -1, (WIDTH as i32 + x) as u32)
        } else if x + WIDTH as i32 > MAP_SIZE as i32 * 8 {
            let sub = WIDTH as i32 - (WIDTH as i32 + x - MAP_SIZE as i32 * 8);
            (x, 0, sub as u32)
        } else {
            (x, 0, WIDTH)
        };
        let (s_y, pos_y, height) = if y < 0 {
            (0, y * -1, (HEIGHT as i32 + y) as u32)
        } else if y + HEIGHT as i32 > MAP_SIZE as i32 * 8 {
            let sub = HEIGHT as i32 - (HEIGHT as i32 + y - MAP_SIZE as i32 * 8);
            (y, 0, sub as u32)
        } else {
            (y, 0, HEIGHT)
        };
        canvas
            .copy(
                &map_data.texture,
                Rect::new(s_x, s_y, width, height),
                Rect::new(pos_x, pos_y, width, height),
            )
            .expect("copy map failed");

        let width = WIDTH / 2 - TILE_WIDTH / 2;
        let height = HEIGHT / 2 - TILE_HEIGHT / 2;
        let (x, y) = player.action.get_current(player.is_running);
        canvas
            .copy(
                &texture,
                Rect::new(x, y, TILE_WIDTH, TILE_HEIGHT),
                Rect::new(width as _, height as _, TILE_WIDTH, TILE_HEIGHT),
            )
            .expect("copy character failed");
        player.apply_move(&map_data);
        hud.draw(&player, &mut canvas);
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
