extern crate sdl2;

use rand_chacha::ChaCha8Rng;
use rand::{Rng, SeedableRng};
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
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MAP_SIZE: u32 = 1_000;

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[repr(usize)]
enum Direction {
    Front = 0,
    Left = 1,
    Right = 2,
    Back = 3,
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
struct Action {
    direction: Direction,
    secondary: Option<Direction>,
    movement: Option<u64>,
}

impl Action {
    /// Returns `(x, y, width, height)`.
    fn get_current(&self, is_running: bool, textures: &TextureHandler<'_>) -> (i32, i32, i32, i32) {
        if let Some(ref pos) = self.movement {
            let (info, nb_animations) = &textures.actions_moving[self.direction as usize];
            let pos = if is_running {
                (pos % 30) as i32 / (30 / nb_animations)
            } else {
                (pos % 60) as i32 / (60 / nb_animations)
            };
            (pos * info.incr_to_next + info.x, info.y, info.width() as i32, info.height() as i32)
        } else {
            let info = &textures.actions_standing[self.direction as usize];
            (info.x, info.y, info.width() as i32, info.height() as i32)
        }
    }
}

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

fn handle_move(player: &mut Player, dir: Direction) {
    if player.character.action.movement.is_none() {
        player.character.action.direction = dir;
        player.character.action.movement = Some(0);
        player.is_running = player.is_run_pressed && player.character.stamina > 0;
    } else if player.character.action.secondary.is_none() && dir != player.character.action.direction {
        player.character.action.secondary = Some(dir);
    }
}

fn handle_release(player: &mut Player, dir: Direction) {
    if Some(dir) == player.character.action.secondary {
        player.character.action.secondary = None;
    } else if dir == player.character.action.direction {
        if let Some(second) = player.character.action.secondary.take() {
            player.character.action.movement = Some(0);
            player.character.action.direction = second;
        } else {
            player.character.action.movement = None;
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

impl<'a> Map<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>, rng: &mut ChaCha8Rng) -> Map<'a> {
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

    fn draw(&self, canvas: &mut Canvas<Window>, screen: &Rect) {
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

#[derive(Debug, Clone, PartialEq)]
struct Dimension {
    rect: Rect,
    incr_to_next: i32,
}

impl Dimension {
    fn new(rect: Rect, incr_to_next: i32) -> Dimension {
        Dimension {
            rect,
            incr_to_next,
        }
    }
}

impl Deref for Dimension {
    type Target = Rect;

    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

struct TextureHandler<'a> {
    texture: Texture<'a>,
    actions_standing: Vec<Dimension>,
    /// The second element is the number of "animations".
    actions_moving: Vec<(Dimension, i32)>,
}

struct Character<'a> {
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
    texture_handler: TextureHandler<'a>,
}

impl<'a> Character<'a> {
    fn move_result(&self, dir: Direction) -> ((i32, i32), (i32, i32)) {
        let (info, _) = &self.texture_handler.actions_moving[dir as usize];
        match dir {
            Direction::Front => ((0, 0), (info.height() as i32 / 2, 1)),
            Direction::Back => ((0, 0), (info.height() as i32 / -4, -1)),
            Direction::Left => ((info.width() as i32 / -2, -1), (0, 0)),
            Direction::Right => ((info.width() as i32 / 2, 1), (0, 0)),
        }
    }

    fn inner_apply_move(&mut self, map: &Map) -> bool {
        if self.action.movement.is_none() {
            return false;
        }
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

    fn draw(&mut self, canvas: &mut Canvas<Window>, is_running: bool, screen: &Rect) {
        let (tile_x, tile_y, tile_width, tile_height) =
            self.action.get_current(is_running, &self.texture_handler);
        if (self.x + tile_width < screen.x || self.x > screen.x + screen.width() as i32) &&
            (self.y + tile_height < screen.y || self.y > screen.y + screen.height() as i32) {
            // No need to draw if we don't see the character.
            return;
        }
        canvas
            .copy(
                &self.texture_handler.texture,
                Rect::new(tile_x, tile_y, tile_width as u32, tile_height as u32),
                Rect::new(self.x - screen.x, self.y - screen.y, tile_width as u32, tile_height as u32),
            )
            .expect("copy character failed");

        // We now update the animation!
        if let Some(ref mut pos) = self.action.movement {
            *pos += 1;
        } else {
            if self.stamina < self.total_stamina {
                self.stamina += 1;
            }
            return;
        }
    }
}

struct Player<'a> {
    character: Character<'a>,
    is_running: bool,
    is_run_pressed: bool,
}

impl<'a> Player<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Player<'a> {
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
                texture_handler,
            },
            is_running: false,
            is_run_pressed: false,
        }
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>, screen: &Rect) {
        self.character.draw(canvas, self.is_running, screen)
    }

    fn apply_move(&mut self, map: &Map) {
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

    fn draw(&self, player: &Player, canvas: &mut Canvas<Window>) {
        macro_rules! draw_bar {
            ($total:ident, $current:ident, $height:expr, $name:expr, $y:expr, $texture:ident) => {{
                let show = 144 * player.character.$current / player.character.$total;
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
    let mut hasher = DefaultHasher::new();
    "hello!".hash(&mut hasher);
    let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let _sdl_img_context = image::init(image::InitFlag::PNG).expect("failed to init SDL image");
    let video_subsystem = sdl_context.video().expect("failed to get video context");

    let window = video_subsystem
        .window("toy game", WIDTH as u32, HEIGHT as u32)
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

    let mut event_pump = sdl_context.event_pump().expect("failed to get event pump");
    let map = Map::new(&texture_creator, &mut rng);
    let mut player = Player::new(&texture_creator);
    let hud = HUD::new(&texture_creator);
    let mut screen = Rect::new(
        player.character.x - WIDTH / 2,
        player.character.y - HEIGHT / 2,
        WIDTH as u32,
        HEIGHT as u32,
    );

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
                        player.is_running = player.character.action.movement.is_some();
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

        player.apply_move(&map);
        // For now, the screen follows the player.
        screen.x = player.character.x - WIDTH / 2;
        screen.y = player.character.y - HEIGHT / 2;
        map.draw(&mut canvas, &screen);
        player.draw(&mut canvas, &screen);
        hud.draw(&player, &mut canvas);

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
