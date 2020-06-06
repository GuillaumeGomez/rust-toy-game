use sdl2::pixels::Color;

use crate::system::System;
use crate::ONE_SECOND;

const STATUS_UPDATE_TIME: u64 = ONE_SECOND / 60;

pub struct Status {
    text: String,
    // When it reaches y_limit, the status should be removed.
    y_pos: i32,
    y_limit: i32,
    duration: u64,
    color: Color,
}

impl<'a> Status {
    pub fn new(text: String, color: Color) -> Status {
        Status {
            y_pos: 0,
            y_limit: 30,
            duration: 0,
            text,
            color,
        }
    }

    pub fn update(&mut self, elapsed: u64) {
        self.duration += elapsed;
        while self.duration > STATUS_UPDATE_TIME && self.y_pos < self.y_limit {
            self.duration -= STATUS_UPDATE_TIME;
            self.y_pos += 1;
        }
    }

    pub fn draw(&self, system: &mut System, x: i64, y: i64) {
        // increase position of the text
        let x = (x - system.x()) as i32;
        let y = (y - system.y()) as i32 - self.y_pos - 10;
        system.draw_text(&self.text, 14, self.color, x, y, true);
    }

    pub fn should_be_removed(&self) -> bool {
        self.y_pos >= self.y_limit
    }
}
