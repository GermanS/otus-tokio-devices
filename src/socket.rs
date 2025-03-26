use std::{error::Error, fmt::Display, str::FromStr};

use regex::Regex;

use crate::power::Power;

#[derive(Debug)]
struct Socket {
    power: Power,
}

impl Socket {
    pub fn new(power: Power) -> Self {
        Self { power }
    }

    pub fn power(&self) -> &Power {
        &self.power
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

mod test {
    use super::*;

    #[test]
    fn positive_f32_in_string() {
        let message = "Socket 21.5 W";

        let socket = Socket::from_str(message);

        assert!(socket.is_ok(), "Looks like string has been parsed well");
        assert!(socket.unwrap().power().get() == 21.5, "Power is correct");
    }

    #[test]
    fn positive_u32_in_message() {
        let message = "Socket  1500 W";

        let result = Socket::from_str(message);

        assert!(result.is_ok(), "Looks like string has been parsed well");
        assert!(result.unwrap().power().get() == 1500.0, "Power is correct");
    }

    #[test]
    fn negative_missing_temperature() {
        let message = "Socket -x- W";

        let result = Socket::from_str(message);

        assert!(!result.is_ok(), "Got an error");
    }
}
