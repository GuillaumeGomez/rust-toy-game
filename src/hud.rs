use sdl2::image::LoadSurface;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::player::Player;
use crate::system::System;

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

pub struct HUD<'a> {
    bars: Texture<'a>,
    bars_width: u32,
    bars_height: u32,
    health_bar: Texture<'a>,
    mana_bar: Texture<'a>,
    stamina_bar: Texture<'a>,
    xp_bar: Texture<'a>,
}

impl<'a> HUD<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> HUD<'a> {
        let bars =
            Surface::from_file("resources/bars.png").expect("failed to load `resources/bars.png`");
        let bars_width = bars.width();
        let bars_height = bars.height();
        let bars = texture_creator
            .create_texture_from_surface(bars)
            .expect("failed to build texture from bars surface");

        let health_bar = create_bar(
            "health bar",
            144,
            4,
            Color::RGB(247, 0, 43),
            texture_creator,
        );
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

    pub fn draw(&self, player: &Player, system: &mut System) {
        macro_rules! draw_bar {
            ($total:ident, $current:ident, $height:expr, $name:expr, $y:expr, $texture:ident) => {{
                let show = 144 * player.character.$current / player.character.$total;
                system
                    .canvas
                    .copy(
                        &self.$texture,
                        Rect::new(0, 0, show, $height),
                        Rect::new(2, $y, show, $height),
                    )
                    .expect(concat!("copy ", $name, " bar failed"));
            }};
        }
        system
            .canvas
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
