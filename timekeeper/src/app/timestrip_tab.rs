use std::{cell::RefCell, rc::Rc};

use crate::{
    app::TimeStrip,
    app::utils::{HIGHLIGHT_SYMBOL, block},
    timestrip::{TimeStamp, TimeStampType},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};

const DATE_FORMAT_STR: &str = "%H:%M:%S.%3f";

pub(crate) struct TimeStripTab {
    state: ListState,

    // shared context
    time_strip: Rc<RefCell<TimeStrip>>,
    selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl Widget for &mut TimeStripTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .time_strip
            .borrow()
            .time_stamps
            .iter()
            .rev()
            .map(ListItem::from)
            .collect();

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
    pub(crate) fn new(
        time_strip: Rc<RefCell<TimeStrip>>,
        selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
        show_time_strip_popup: Rc<RefCell<bool>>,
    ) -> Self {
        Self {
            state: ListState::default(),
            time_strip,
            selected_time_stamp,
            show_time_strip_popup,
        }
    }

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
                    *self.show_time_strip_popup.borrow_mut() = true;
                }
            }
            _ => {}
        }
        self.update_selected_time_stamp();
    }

    fn update_selected_time_stamp(&mut self) {
        let time_stamps = &self.time_strip.borrow().time_stamps;

        // get the index of the selected time stamp
        if let Some(index) = self.state.selected() {
            // as the list is reversed, we need to calculate the correct index in the time strip
            let time_strip_index = time_stamps.len().saturating_sub(index).saturating_sub(1);
            // get the time stamp from the time strip
            if let Some(time_stamp) = time_stamps.get(time_strip_index) {
                *self.selected_time_stamp.borrow_mut() = Some(time_stamp.clone());
            } else {
                *self.selected_time_stamp.borrow_mut() = None;
            }
        } else {
            *self.selected_time_stamp.borrow_mut() = None;
        }
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
