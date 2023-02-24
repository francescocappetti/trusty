#![allow(clippy::len_without_is_empty)]

mod document;
mod editor;
mod position;
mod row;
mod size;
mod status_message;
mod terminal;

pub use document::Document;
use editor::Editor;
pub use position::Position;
pub use row::Row;
pub use size::Size;
pub use status_message::StatusMessage;
pub use terminal::Terminal;

fn main() -> crossterm::Result<()> {
    Editor::default().run()
}
