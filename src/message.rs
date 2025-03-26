pub enum ThermometerMessage {
    Off,
    Value(f32),
}

impl From<String> for ThermometerMessage {
    fn from(value: String) -> Self {
        let v = value.trim().parse::<f32>().unwrap();

        match v {
            0.0 => ThermometerMessage::Off,
            _ => ThermometerMessage::Value(v),
        }
    }
}

impl From<ThermometerMessage> for String {
    fn from(value: ThermometerMessage) -> Self {
        match value {
            ThermometerMessage::Off => "0".into(),
            ThermometerMessage::Value(v) => format!("{}", v),
        }
    }
}

pub enum SocketMessage {
    Off,
    Value(u8),
}

impl From<String> for SocketMessage {
    fn from(value: String) -> Self {
        let v = value.trim().parse::<u8>().unwrap();

        match v {
            0 => SocketMessage::Off,
            _ => SocketMessage::Value(v),
        }
    }
}

impl From<SocketMessage> for String {
    fn from(value: SocketMessage) -> Self {
        match value {
            SocketMessage::Off => "0".into(),
            SocketMessage::Value(v) => format!("{}", v),
        }
    }
}
