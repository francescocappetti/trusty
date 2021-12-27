use crate::Document;
use crate::Position;
use crate::StatusMessage;
use crate::Terminal;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::env;
use std::time::{Duration, Instant};

const FG_COLOR: Color = Color::White;
const BG_COLOR: Color = Color::Black;
const STATUS_BAR_FG_COLOR: Color = Color::Black;
const STATUS_BAR_BG_COLOR: Color = Color::White;
const TILDES_COLOR: Color = Color::DarkGrey;
const TAB_SIZE: usize = 4;
const STATUS_MESSAGE_DURATION: u64 = 5;

pub struct Editor {
    running: bool,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
}

impl Default for Editor {
    fn default() -> Editor {
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
            status_message: StatusMessage::from(
                "[HELP] CTRL-Q = quit | CTRL-S = save | CTRL-Z = undo | CTRL-Y = redo",
            ),
        }
    }
}

impl Editor {
    pub fn run(&mut self) -> crossterm::Result<()> {
        Terminal::enter_alternate_screen()?;
        Terminal::enable_raw_mode()?;

        while self.running {
            self.refresh_screen()?;
            self.process_event()?;
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

    fn process_event(&mut self) -> crossterm::Result<()> {
        if let Event::Key(key) = Terminal::read_event()? {
            match key {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    if self.document.is_dirty() {
                        if matches!(
                            &self.prompt("Are you sure you want to quit without saving? [Y/n]")?[..],
                            "Y"
                        ) {
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
                        self.document.filename = Some(self.prompt("Save As:")?);
                        self.document.save()?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('z'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    // if let Some(position) = self.history.undo(&mut self.document) {
                    //     self.cursor_position = position;
                    // }
                }
                KeyEvent {
                    code: KeyCode::Char('y'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    // if let Some(position) = self.history.redo(&mut self.document) {
                    //     self.cursor_position = position;
                    // }
                }
                KeyEvent {
                    code: direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right),
                    modifiers: KeyModifiers::NONE,
                } => {
                    self.move_cursor(direction)?;
                }
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
                    for _ in 0..TAB_SIZE {
                        self.document.insert(' ', &self.cursor_position);
                        self.move_cursor(KeyCode::Right)?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Delete,
                    modifiers: KeyModifiers::NONE,
                } => {
                    self.document.delete(&self.cursor_position);
                }
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
        }

        self.scroll()
    }

    fn draw_rows(&self) -> crossterm::Result<()> {
        let width = Terminal::size()?.width;
        let height = Terminal::size()?.height;

        Terminal::set_bg_color(BG_COLOR)?;

        for i in 0..height {
            Terminal::clear_current_line()?;

            if let Some(row) = self.document.row(i + self.offset.y) {
                let row = row.render(self.offset.x as usize, (self.offset.x + width) as usize);

                Terminal::set_fg_color(FG_COLOR)?;
                println!("{}\r", row);
            } else {
                Terminal::set_fg_color(TILDES_COLOR)?;
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

        let dirty = if self.document.is_dirty() {
            " [+] "
        } else {
            ""
        };

        let position = format!("{}:{}", self.cursor_position.y, self.cursor_position.x);

        let spaces =
            if let Some(n) = width.checked_sub(filename.len() + dirty.len() + position.len()) {
                " ".repeat(n)
            } else {
                String::new()
            };

        let mut bar = format!("{}{}{}{}", filename, dirty, spaces, position);
        bar.truncate(width);

        Terminal::set_bg_color(STATUS_BAR_BG_COLOR)?;
        Terminal::set_fg_color(STATUS_BAR_FG_COLOR)?;
        println!("{}\r", bar);
        Terminal::reset_color()
    }

    fn draw_status_message(&self) -> crossterm::Result<()> {
        Terminal::clear_current_line()?;

        if Instant::now() - self.status_message.time < Duration::new(STATUS_MESSAGE_DURATION, 0) {
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

            if let Event::Key(key) = Terminal::read_event()? {
                match key {
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
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        if !result.is_empty() {
                            result.pop();

                            if cursor_position.x > prompt.len() as u16 + 1 {
                                cursor_position.x -= 1;
                            } else {
                                offset -= 1;
                            }
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.status_message = StatusMessage::default();
                        break;
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }
}
