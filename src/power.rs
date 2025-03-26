use std::fmt::Display;

#[derive(Debug, Default)]
pub struct Power {
    power: f32,
}

impl Power {
    pub fn new(power: f32) -> Self {
        Self { power }
    }

    pub fn get(&self) -> f32 {
        self.power
    }

    pub fn set(&mut self, value: f32) {
        self.power = value
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.power)
    }
}
