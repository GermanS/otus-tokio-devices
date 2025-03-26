use std::fmt::Display;

#[derive(Debug, Default)]
pub struct Temperature {
    value: f32,
}

impl Temperature {
    pub fn new(temperature: f32) -> Self {
        Self { value: temperature }
    }

    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.value)
    }
}
