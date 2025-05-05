use std::{error::Error, fmt::Display, str::FromStr};

use regex::Regex;

use crate::power::Power;

#[derive(Debug, Default)]
pub struct Socket {
    power: Power,
}

impl Socket {
    pub fn new(power: Power) -> Self {
        Self { power }
    }

    pub fn power(&self) -> &Power {
        &self.power
    }

    pub fn power_mut(&mut self) -> &mut Power {
        &mut self.power
    }
}

impl Display for Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Socket {} W", self.power)
    }
}

impl FromStr for Socket {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^Socket(\s)+(\d+(\.\d+)?)").unwrap();

        match re.captures(s) {
            Some(caps) => {
                if let Ok(t) = caps[2].parse::<f32>() {
                    return Ok(Self::new(Power::new(t)));
                }

                Err("cannot parse float from string".into())
            }
            None => Err("does not look like message from socket".into()),
        }
    }
}
