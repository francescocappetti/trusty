use crate::Document;
use crate::Position;
use crate::StatusMessage;
use crate::Terminal;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::env;
use std::time::{Duration, Instant};

pub struct Editor {
    running: bool,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
}

impl Editor {
    pub fn default() -> Editor {
        let args: Vec<String> = env::args().collect();

        let document = if args.len() > 1 {
            let filename = &args[1];
            Document::open(filename)
        } else {
            Document::default()
        };

        Editor {
            running: true,
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from("[HELP] CTRL-Q = quit | CTRL-S = save"),
        }
    }

    pub fn run(&mut self) -> crossterm::Result<()> {
        Terminal::enter_alternate_screen()?;
        Terminal::enable_raw_mode()?;

        while self.running {
            self.refresh_screen()?;
            self.process_keypress()?;
        }

        Terminal::disable_raw_mode()?;
        Terminal::leave_alternate_screen()
    }

    fn refresh_screen(&self) -> crossterm::Result<()> {
        Terminal::cursor_hide()?;
        Terminal::cursor_position(&Position::default())?;

        self.draw_rows()?;
        self.draw_status_bar()?;
        self.draw_status_message()?;

        Terminal::cursor_position(&Position::new(
            self.cursor_position.x.saturating_sub(self.offset.x),
            self.cursor_position.y.saturating_sub(self.offset.y),
        ))?;
        Terminal::cursor_show()?;

        Terminal::flush()
    }

    fn process_keypress(&mut self) -> crossterm::Result<()> {
        match Terminal::read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                if self.document.is_dirty() {
                    if self.prompt("Are you sure you want to quit without saving? [Y/n]")? == "Y" {
                        self.running = false;
                    } else {
                        self.status_message = StatusMessage::from("[WARNING] File not saved");
                    }
                } else {
                    self.running = false;
                }
            }
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                if self.document.filename.is_some() {
                    self.document.save()?;
                } else {
                    self.document.filename = Some(self.prompt("Save as:")?);
                    self.document.save()?;
                }
            }
            KeyEvent {
                code: direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right),
                modifiers: KeyModifiers::NONE,
            } => self.move_cursor(direction)?,
            KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => {
                self.document.insert(ch, &self.cursor_position);
                self.move_cursor(KeyCode::Right)?;
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            } => {
                self.document.insert_newline(&self.cursor_position);
                self.move_cursor(KeyCode::Down)?;
                self.cursor_position.x = 0;
            }
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
            } => {
                for _ in 0..4 {
                    self.document.insert(' ', &self.cursor_position);
                    self.move_cursor(KeyCode::Right)?;
                }
            }
            KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
            } => self.document.delete(&self.cursor_position),
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            } => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(KeyCode::Left)?;
                    self.document.delete(&self.cursor_position);
                }
            }
            _ => {}
        }

        self.scroll()
    }

    fn draw_rows(&self) -> crossterm::Result<()> {
        let width = Terminal::size()?.width;
        let height = Terminal::size()?.height;

        for i in 0..height {
            Terminal::clear_current_line()?;

            if let Some(row) = self.document.row(i + self.offset.y) {
                let row = row.render(self.offset.x as usize, (self.offset.x + width) as usize);

                Terminal::set_fg_color(Color::White)?;
                println!("{}\r", row);
            } else {
                Terminal::set_fg_color(Color::DarkGrey)?;
                println!("~\r");
            }
        }

        Ok(())
    }

    fn draw_status_bar(&self) -> crossterm::Result<()> {
        let width = Terminal::size()?.width as usize;

        let filename = if let Some(filename) = &self.document.filename {
            filename
        } else {
            "[No Name]"
        };
        let position = format!("{}:{}", self.cursor_position.y, self.cursor_position.x);
        let dirty = if self.document.is_dirty() {
            " [+] "
        } else {
            ""
        };
        let spaces = " ".repeat(width - filename.len() - dirty.len() - position.len());

        let mut bar = format!("{}{}{}{}", filename, dirty, spaces, position);
        bar.truncate(width);

        Terminal::set_bg_color(Color::White)?;
        Terminal::set_fg_color(Color::Black)?;
        println!("{}\r", bar);
        Terminal::reset_color()
    }

    fn draw_status_message(&self) -> crossterm::Result<()> {
        Terminal::clear_current_line()?;

        if Instant::now() - self.status_message.time < Duration::new(5, 0) {
            let mut message = self.status_message.text.clone();
            message.truncate(Terminal::size()?.width as usize);

            print!("{}", message);
        }

        Ok(())
    }

    fn move_cursor(&mut self, direction: KeyCode) -> crossterm::Result<()> {
        let width = if let Some(row) = self.document.row(self.cursor_position.y) {
            row.len() as u16
        } else {
            0
        };
        let height = self.document.len() as u16;

        match direction {
            KeyCode::Up => {
                if self.cursor_position.y > 0 {
                    self.cursor_position.y -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_position.y < height {
                    self.cursor_position.y += 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_position.x > 0 {
                    self.cursor_position.x -= 1;
                } else if self.cursor_position.y > 0 {
                    self.cursor_position.y -= 1;
                    self.cursor_position.x =
                        self.document.row(self.cursor_position.y).unwrap().len() as u16;
                }
            }
            KeyCode::Right => {
                if self.cursor_position.x < width {
                    self.cursor_position.x += 1;
                } else if self.cursor_position.y < height {
                    self.cursor_position.y += 1;
                    self.cursor_position.x = 0;
                }
            }
            _ => {}
        }

        let width = if let Some(row) = self.document.row(self.cursor_position.y) {
            row.len() as u16
        } else {
            0
        };

        if self.cursor_position.x > width {
            self.cursor_position.x = width;
        }

        Ok(())
    }

    fn scroll(&mut self) -> crossterm::Result<()> {
        let width = Terminal::size()?.width;
        let height = Terminal::size()?.height;

        if self.cursor_position.x < self.offset.x {
            self.offset.x = self.cursor_position.x;
        } else if self.cursor_position.x >= self.offset.x.saturating_add(width) {
            self.offset.x = self
                .cursor_position
                .x
                .saturating_sub(width)
                .saturating_add(1);
        }

        if self.cursor_position.y < self.offset.y {
            self.offset.y = self.cursor_position.y;
        } else if self.cursor_position.y >= self.offset.y.saturating_add(height) {
            self.offset.y = self
                .cursor_position
                .y
                .saturating_sub(height)
                .saturating_add(1);
        }

        Ok(())
    }

    fn prompt(&mut self, prompt: &str) -> crossterm::Result<String> {
        let mut result = String::new();

        let mut cursor_position =
            Position::new(prompt.len() as u16 + 1, Terminal::size()?.height + 1);
        let mut offset = 0;

        loop {
            self.status_message =
                StatusMessage::from(&format!("{} {}", prompt, &result[offset..result.len()])[..]);
            self.refresh_screen()?;
            Terminal::cursor_position(&cursor_position)?;

            match Terminal::read_key()? {
                KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                } => {
                    result.push(ch);

                    if cursor_position.x < Terminal::size()?.width - 1 {
                        cursor_position.x += 1;
                    } else {
                        offset += 1;
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    self.status_message = StatusMessage::new();
                    break;
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if result.len() > 0 {
                        result.pop();

                        if cursor_position.x > prompt.len() as u16 + 1 {
                            cursor_position.x -= 1;
                        } else {
                            offset -= 1;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(result)
    }
}
