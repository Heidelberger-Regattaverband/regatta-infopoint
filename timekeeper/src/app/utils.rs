use ratatui::{
    symbols::border,
    widgets::{Block, BorderType, Padding},
};

pub(crate) const HIGHLIGHT_SYMBOL: &str = ">>  ";

/// A block surrounding the tab's content
pub(crate) fn block() -> Block<'static> {
    Block::bordered()
        .border_set(border::PROPORTIONAL_TALL)
        .padding(Padding::horizontal(1))
}

/// A block surrounding the popup's content
pub(crate) fn popup_block() -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
}
