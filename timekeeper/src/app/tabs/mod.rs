pub(super) mod logs;
pub(super) mod measurement;
pub(super) mod timestrip;

use ratatui::{
    symbols::border,
    widgets::{Block, Padding},
};

/// A block surrounding the tab's content
fn block() -> Block<'static> {
    Block::bordered()
        .border_set(border::PROPORTIONAL_TALL)
        .padding(Padding::horizontal(1))
}
