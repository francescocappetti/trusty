use std::time::Instant;

pub struct StatusMessage {
    pub text: String,
    pub time: Instant,
}

impl From<&str> for StatusMessage {
    fn from(message: &str) -> StatusMessage {
        StatusMessage {
            text: message.to_string(),
            time: Instant::now(),
        }
    }
}

impl StatusMessage {
    pub fn new() -> StatusMessage {
        StatusMessage {
            text: String::new(),
            time: Instant::now(),
        }
    }
}
