use crate::{
    app::{
        tabs::{block, popup_block, HIGHLIGHT_SYMBOL},
        TimeStrip,
    },
    timestrip::{TimeStamp, TimeStampType},
};
use crossterm::event::{KeyCode, KeyEvent};
use log::warn;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};
use tui_textarea::{Input, TextArea};

const DATE_FORMAT_STR: &str = "%H:%M:%S.%3f";

#[derive(Default)]
pub(crate) struct TimeStripTab {
    pub(crate) time_strip: TimeStrip,
    state: ListState,
    pub(crate) show_popup: bool,
}

impl Widget for &mut TimeStripTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.time_strip.time_stamps.iter().rev().map(ListItem::from).collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block())
            .highlight_symbol(HIGHLIGHT_SYMBOL)
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

impl TimeStripTab {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => self.state.select_next(),
            KeyCode::Home => self.state.select_first(),
            KeyCode::End => self.state.select_last(),
            KeyCode::Char('h') => self.state.select(None),
            KeyCode::Enter => {
                // open popup if a time stamp is selected
                if self.state.selected().is_some() {
                    self.show_popup = !self.show_popup;
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub(crate) struct TimeStripTabPopup<'a> {
    input: TextArea<'a>,
}

impl Widget for &mut TimeStripTabPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut textarea = TextArea::default();
        textarea.set_block(popup_block().title(" Lauf "));
        textarea.set_cursor_line_style(Style::default());
        textarea.render(area, buf);
    }
}

impl TimeStripTabPopup<'_> {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        let input: Input = event.into();
        warn!("Input {:?}", input);
        self.input.input(input);
    }
}

impl From<&TimeStamp> for ListItem<'_> {
    fn from(value: &TimeStamp) -> Self {
        match value.stamp_type {
            TimeStampType::Start => ListItem::new(format!(
                "Start {:4}:  {}  {:3}  {:2}",
                value.index,
                value.time.format(DATE_FORMAT_STR),
                value.heat_nr.unwrap_or(0),
                value.bib.unwrap_or(0)
            )),
            TimeStampType::Finish => ListItem::new(format!(
                " Ziel {:4}:  {}  {:3}  {:2}",
                value.index,
                value.time.format(DATE_FORMAT_STR),
                value.heat_nr.unwrap_or(0),
                value.bib.unwrap_or(0)
            )),
        }
    }
}
