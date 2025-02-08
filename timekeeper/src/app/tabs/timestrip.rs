use crate::{
    app::{tabs::block, TimeStrip},
    timestrip::{TimeStamp, TimeStampType},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        palette::tailwind::{GREEN, SLATE},
        Color,
    },
    text::Line,
    widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};

const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;
const DATE_FORMAT_STR: &str = "%H:%M:%S.%3f";

#[derive(Default)]
pub(crate) struct TimeStripTab {
    // Add fields here
    time_strip: TimeStrip,
    state: ListState,
}

impl Widget for &mut TimeStripTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.time_strip.time_stamps.iter().rev().map(ListItem::from).collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block())
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

impl TimeStripTab {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('s') | KeyCode::Char('+') => {
                self.start_time_stamp();
                true
            }
            KeyCode::Char('f') | KeyCode::Char(' ') => {
                self.finish_time_stamp();
                true
            }
            _ => false,
        }
    }

    fn finish_time_stamp(&mut self) {
        self.time_strip.add_new_finish();
    }

    fn start_time_stamp(&mut self) {
        self.time_strip.add_new_start();
    }
}

impl From<&TimeStamp> for ListItem<'_> {
    fn from(value: &TimeStamp) -> Self {
        let line = match value.stamp_type {
            TimeStampType::Start => Line::styled(
                format!(
                    "Start {:4}:  {}  {:3}  {:2}",
                    value.index,
                    value.time.format(DATE_FORMAT_STR),
                    value.heat_nr.unwrap_or(0),
                    value.bib.unwrap_or(0)
                ),
                TEXT_FG_COLOR,
            ),
            TimeStampType::Finish => Line::styled(
                format!(
                    " Ziel {:4}:  {}  {:3}  {:2}",
                    value.index,
                    value.time.format(DATE_FORMAT_STR),
                    value.heat_nr.unwrap_or(0),
                    value.bib.unwrap_or(0)
                ),
                COMPLETED_TEXT_FG_COLOR,
            ),
        };
        ListItem::new(line)
    }
}
