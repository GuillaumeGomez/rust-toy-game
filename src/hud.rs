use crate::sdl2::image::LoadSurface;
use crate::sdl2::pixels::Color;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::{Texture, TextureCreator};
use crate::sdl2::surface::Surface;
use crate::sdl2::video::WindowContext;

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

    fn draw_bar(&self, system: &mut System, value: u32, height: u32, y: i32, texture: &Texture) {
        system
            .canvas
            .copy(
                texture,
                Rect::new(0, 0, value, height),
                Rect::new(2, y, value, height),
            )
            .expect("copy bar failed");
    }

    pub fn draw(&self, player: &Player, system: &mut System) {
        system
            .canvas
            .copy(
                &self.bars,
                None,
                Rect::new(0, 0, self.bars_width, self.bars_height),
            )
            .expect("copy hud failed");
        self.draw_bar(
            system,
            144 * player.character.stats.health.value() as u32
                / player.character.stats.health.max_value() as u32,
            4,
            2,
            &self.health_bar,
        );
        self.draw_bar(
            system,
            144 * player.character.stats.mana.value() as u32
                / player.character.stats.mana.max_value() as u32,
            4,
            8,
            &self.mana_bar,
        );
        self.draw_bar(
            system,
            144 * player.character.stats.stamina.value() as u32
                / player.character.stats.stamina.max_value() as u32,
            4,
            14,
            &self.stamina_bar,
        );
        self.draw_bar(
            system,
            (144 * player.character.xp / player.character.xp_to_next_level) as u32,
            2,
            20,
            &self.xp_bar,
        );
    }
}
