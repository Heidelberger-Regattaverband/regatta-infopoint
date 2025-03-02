use ratatui::{
    symbols::border,
    widgets::{Block, Padding},
};

pub(crate) const HIGHLIGHT_SYMBOL: &str = ">>  ";

/// A block surrounding the tab's content
pub(crate) fn block() -> Block<'static> {
    Block::bordered()
        .border_set(border::PROPORTIONAL_TALL)
        .padding(Padding::horizontal(1))
}
