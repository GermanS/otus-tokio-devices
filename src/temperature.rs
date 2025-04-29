use std::fmt::Display;

#[derive(Debug, Default)]
pub struct Temperature(f32);

impl Temperature {
    pub const MIN_TEMPERATURE: f32 = 0.0;
    pub const MAX_TEMPERATURE: f32 = 100.0;
    pub const GRADUATION: f32 = 0.5;

    pub fn new(temperature: f32) -> Self {
        Self(temperature)
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn set(&mut self, value: f32) {
        if (Self::MIN_TEMPERATURE..=Self::MAX_TEMPERATURE).contains(&value) {
            self.0 = value
        }
    }

    pub fn ratio(temperature: f32) -> f32 {
        (temperature - Self::MIN_TEMPERATURE) / (Self::MAX_TEMPERATURE - Self::MIN_TEMPERATURE)
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.get())
    }
}
