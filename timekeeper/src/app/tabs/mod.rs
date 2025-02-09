pub(super) mod heats;
pub(super) mod logs;
pub(super) mod timestrip;

use ratatui::{
    symbols::border,
    text::Line,
    widgets::{Block, BorderType, Padding},
};
use strum::{Display, EnumIter, FromRepr};

const HIGHLIGHT_SYMBOL: &str = ">>  ";

/// A block surrounding the tab's content
fn block() -> Block<'static> {
    Block::bordered()
        .border_set(border::PROPORTIONAL_TALL)
        .padding(Padding::horizontal(1))
}

/// A block surrounding the popup's content
fn popup_block() -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub(super) enum SelectedTab {
    #[default]
    #[strum(to_string = "Läufe")]
    Heats,
    #[strum(to_string = "Zeitstreifen")]
    TimeStrip,
    #[strum(to_string = "Logs")]
    Logs,
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    pub(super) fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    pub(super) fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    /// Return tab's name as a styled `Line`
    pub(super) fn title(self) -> Line<'static> {
        format!("  {self}  ").into()
    }
}
