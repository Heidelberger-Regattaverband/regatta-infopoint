use ratatui::text::Line;
use strum_macros::{Display, EnumIter, FromRepr};

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
        let next_index = (current_index + 1) % 3; // 3 is the number of tabs
        Self::from_repr(next_index).unwrap_or(self)
    }

    /// Return tab's name as a styled `Line`
    pub(super) fn title(self) -> Line<'static> {
        format!("  {self}  ").into()
    }
}
