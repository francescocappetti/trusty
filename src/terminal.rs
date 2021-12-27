use crate::Position;
use crate::Size;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{self, Write};

pub struct Terminal;

impl Terminal {
    pub fn read_event() -> crossterm::Result<Event> {
        loop {
            if let Ok(event) = event::read() {
                return Ok(event);
            }
        }
    }

    pub fn enter_alternate_screen() -> crossterm::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)
    }

    pub fn leave_alternate_screen() -> crossterm::Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)
    }

    pub fn enable_raw_mode() -> crossterm::Result<()> {
        terminal::enable_raw_mode()
    }

    pub fn disable_raw_mode() -> crossterm::Result<()> {
        terminal::disable_raw_mode()
    }

    pub fn size() -> crossterm::Result<Size> {
        let (width, height) = terminal::size()?;
        Ok(Size::new(width, height - 2))
    }

    pub fn clear_all() -> crossterm::Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))
    }

    pub fn clear_current_line() -> crossterm::Result<()> {
        execute!(io::stdout(), Clear(ClearType::CurrentLine))
    }

    pub fn cursor_hide() -> crossterm::Result<()> {
        execute!(io::stdout(), Hide)
    }

    pub fn cursor_show() -> crossterm::Result<()> {
        execute!(io::stdout(), Show)
    }

    pub fn cursor_position(position: &Position) -> crossterm::Result<()> {
        execute!(io::stdout(), MoveTo(position.x, position.y))
    }

    pub fn set_bg_color(color: Color) -> crossterm::Result<()> {
        execute!(io::stdout(), SetBackgroundColor(color))
    }

    pub fn set_fg_color(color: Color) -> crossterm::Result<()> {
        execute!(io::stdout(), SetForegroundColor(color))
    }

    pub fn reset_color() -> crossterm::Result<()> {
        execute!(io::stdout(), ResetColor)
    }

    pub fn flush() -> crossterm::Result<()> {
        io::stdout().flush()
    }
}
