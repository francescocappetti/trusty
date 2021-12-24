use std::{cmp, fmt};

pub struct Row {
    string: String,
}

impl From<&str> for Row {
    fn from(string: &str) -> Row {
        Row {
            string: String::from(string),
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl Row {
    pub fn default() -> Row {
        Row {
            string: String::new(),
        }
    }

    pub fn render(&self, start: usize, end: usize) -> &str {
        &self.string[cmp::min(start, self.len())..cmp::min(end, self.len())]
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }

    pub fn insert(&mut self, ch: char, at: usize) {
        self.string.insert(at, ch);
    }

    pub fn delete(&mut self, at: usize) {
        if self.string.chars().nth(at).is_some() {
            self.string.remove(at);
        }
    }

    pub fn append(&mut self, row: &Row) {
        self.string = format!("{}{}", self.string, row);
    }

    pub fn split(&mut self, at: usize) -> Row {
        let (beginning, remainder) = self.string.split_at(at);
        let row = Row::from(remainder);
        self.string = beginning.to_string();

        row
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
}
