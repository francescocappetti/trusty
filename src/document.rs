use crate::Position;
use crate::Row;
use std::io::Write;
use std::{cmp, fs};

#[derive(Default, Clone)]
pub struct Document {
    pub filename: Option<String>,
    rows: Vec<Row>,
    dirty: bool,
}

impl Document {
    pub fn open(filename: &str) -> Document {
        let rows: Vec<Row> = if let Ok(contents) = fs::read_to_string(filename) {
            contents.lines().map(Row::from).collect()
        } else {
            Vec::new()
        };

        Document {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn row(&self, i: u16) -> Option<&Row> {
        self.rows.get(i as usize)
    }

    pub fn insert(&mut self, ch: char, position: &Position) {
        self.dirty = true;

        if let Some(row) = self.rows.get_mut(position.y as usize) {
            let at = cmp::min(position.x as usize, row.len());
            row.insert(ch, at);
        } else {
            let mut row = Row::default();
            row.insert(ch, 0);
            self.rows.push(row);
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() as u16 {
            return;
        }

        self.dirty = true;

        if at.y == self.len() as u16 {
            self.rows.push(Row::default());
        } else {
            let row = self
                .rows
                .get_mut(at.y as usize)
                .unwrap()
                .split(at.x as usize);
            self.rows.insert((at.y + 1) as usize, row);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        if at.y >= self.len() as u16 {
            return;
        }

        self.dirty = true;

        if at.x as usize == self.rows[at.y as usize].len() && (at.y as usize) < self.len() - 1 {
            let next = self.rows.remove((at.y as usize) + 1);
            let row = self.rows.get_mut(at.y as usize).unwrap();
            row.append(&next);
        } else {
            let row = self.rows.get_mut(at.y as usize).unwrap();
            row.delete(at.x as usize);
        }
    }

    pub fn save(&mut self) -> crossterm::Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;

            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }

        self.dirty = false;

        Ok(())
    }

    pub fn find(&self, query: &str) -> Option<Position> {
        for (y, row) in self.rows.iter().enumerate() {
            if let Some(x) = row.find(query) {
                return Some(Position::new(x as u16, y as u16));
            }
        }

        None
    }
}
