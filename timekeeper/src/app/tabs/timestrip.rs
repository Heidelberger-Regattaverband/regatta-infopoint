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
    time_strip: TimeStrip,
    state: ListState,
    show_popup: bool,
}

impl Widget for &mut TimeStripTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.time_strip.time_stamps.iter().rev().map(ListItem::from).collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block())
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

impl TimeStripTab {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('s') | KeyCode::Char('+') => self.start_time_stamp(),
            KeyCode::Char('f') | KeyCode::Char(' ') => self.finish_time_stamp(),
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Home => self.select_first(),
            KeyCode::End => self.select_last(),
            KeyCode::Char('h') => self.select_none(),
            KeyCode::Enter => self.show_popup = !self.show_popup,
            _ => {}
        }
    }

    fn finish_time_stamp(&mut self) {
        self.time_strip.add_new_finish();
    }

    fn start_time_stamp(&mut self) {
        self.time_strip.add_new_start();
    }

    fn select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
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
