use ratatui::{
    symbols::border,
    widgets::{Block, Padding},
};

pub mod logs;

/// A block surrounding the tab's content
fn block() -> Block<'static> {
    Block::bordered()
        .border_set(border::PROPORTIONAL_TALL)
        .padding(Padding::horizontal(1))
}
