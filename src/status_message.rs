use std::time::Instant;

pub struct StatusMessage {
    pub text: String,
    pub time: Instant,
}

impl From<String> for StatusMessage {
    fn from(message: String) -> StatusMessage {
        StatusMessage {
            text: message,
            time: Instant::now(),
        }
    }
}

impl Default for StatusMessage {
    fn default() -> StatusMessage {
        StatusMessage {
            text: String::new(),
            time: Instant::now(),
        }
    }
}
