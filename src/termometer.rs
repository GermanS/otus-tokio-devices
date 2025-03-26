use std::{error::Error, fmt::Display, str::FromStr};

use regex::Regex;

use crate::temperature::Temperature;

#[derive(Debug)]
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

mod test {
    use super::*;

    #[test]
    fn positive_f32_in_string() {
        let message = "Termometer 21.5 C";

        let termometer = Termometer::from_str(message);

        assert!(termometer.is_ok(), "Looks like string has been parsed well");
        assert!(
            termometer.unwrap().temperature().get() == 21.5,
            "Temperature is correct"
        );
    }

    #[test]
    fn positive_u32_in_message() {
        let message = "Termometer 21 C";

        let termometer = Termometer::from_str(message);

        assert!(termometer.is_ok(), "Looks like string has been parsed well");
        assert!(
            termometer.unwrap().temperature().get() == 21.0,
            "Temperature is correct"
        );
    }

    #[test]
    fn negative_missing_temperature() {
        let message = "Termometer x C";

        let termometer = Termometer::from_str(message);

        assert!(!termometer.is_ok(), "Got an error");
    }
}
