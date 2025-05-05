use std::{error::Error, fmt::Display, str::FromStr};

use regex::Regex;

use crate::temperature::Temperature;

#[derive(Debug, Default)]
pub struct Termometer {
    temperature: Temperature,
}

impl Termometer {
    pub fn new(temperature: Temperature) -> Self {
        Self { temperature }
    }

    pub fn temperature(&self) -> &Temperature {
        &self.temperature
    }

    pub fn temperature_mut(&mut self) -> &mut Temperature {
        &mut self.temperature
    }
}

impl Display for Termometer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Termometer {}", self.temperature())
    }
}

impl FromStr for Termometer {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^Termometer(\s)+(\d+(\.\d+)?)").unwrap();

        match re.captures(s) {
            Some(caps) => {
                if let Ok(t) = caps[2].parse::<f32>() {
                    return Ok(Self::new(Temperature::new(t)));
                }

                Err("cannot parse float from string".into())
            }
            None => Err("does not look like message from termometer".into()),
        }
    }
}
