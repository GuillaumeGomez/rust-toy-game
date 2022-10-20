use crate::ONE_SECOND;

#[derive(Clone, Debug)]
pub struct Stat {
    /// How much got regenerated in a second.
    pub regen_rate: f32,
    max_value: f32,
    value: f32,
}

impl Stat {
    pub fn new(regen_rate: f32, max_value: f32) -> Stat {
        Stat {
            regen_rate,
            max_value,
            value: max_value,
        }
    }

    pub fn add(&mut self, add: f32) {
        self.value += add;
        if self.value > self.max_value {
            self.value = self.max_value
        }
    }

    pub fn subtract(&mut self, sub: f32) {
        if sub > self.value {
            self.value = 0.;
        } else {
            self.value -= sub;
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn max_value(&self) -> f32 {
        self.max_value
    }

    pub fn is_full(&self) -> bool {
        self.value >= self.max_value
    }

    pub fn is_empty(&self) -> bool {
        self.value < 0.1
    }

    pub fn pourcent(&self) -> f32 {
        self.value * 100. / self.max_value
    }

    // Duration.as_secs_f32
    pub fn refresh(&mut self, elapsed: f32) -> bool {
        if self.value >= self.max_value {
            return false;
        }
        self.value += elapsed * self.regen_rate;
        if self.value > self.max_value {
            self.value = self.max_value;
        }
        true
    }

    /// Put back the stat at the max.
    pub fn reset(&mut self) {
        self.value = self.max_value;
    }

    pub fn to_string(&self) -> String {
        format!("{} / {}", self.value, self.max_value)
    }
}
