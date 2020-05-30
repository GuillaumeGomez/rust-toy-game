use crate::ONE_SECOND;

#[derive(Clone)]
pub struct Stat {
    /// How much got regenerated in a thousand second
    pub regen_rate: u64,
    max_value: u64,
    value: u64,
}

impl Stat {
    pub fn new(regen_rate: f32, max_value: u64) -> Stat {
        Stat {
            regen_rate: (regen_rate * 1_000.) as u64,
            max_value: max_value * 1_000,
            value: max_value * 1_000,
        }
    }

    pub fn add(&mut self, add: u64) {
        self.value += add * 1000;
        if self.value > self.max_value {
            self.value = self.max_value
        }
    }

    pub fn subtract(&mut self, sub: u64) {
        let sub = sub * 1_000;
        if sub > self.value {
            self.value = 0;
        } else {
            self.value -= sub;
        }
    }

    pub fn value(&self) -> u64 {
        self.value / 1_000
    }

    pub fn max_value(&self) -> u64 {
        self.max_value / 1_000
    }

    pub fn is_full(&self) -> bool {
        self.value >= self.max_value
    }

    pub fn is_empty(&self) -> bool {
        self.value < 1_000
    }

    pub fn pourcent(&self) -> u32 {
        (self.value * 100 / self.max_value) as u32
    }

    pub fn refresh(&mut self, elapsed: u64) {
        if self.value >= self.max_value {
            return;
        }
        self.value += elapsed / (ONE_SECOND / 1_000) * self.regen_rate / 1_000;
        if self.value > self.max_value {
            self.value = self.max_value;
        }
    }

    /// Put back the stat at the max.
    pub fn reset(&mut self) {
        self.value = self.max_value;
    }
}
