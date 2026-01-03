use crate::app::{
    TimeStrip,
    utils::{HIGHLIGHT_SYMBOL, block},
};
use db::timekeeper::TimeStamp;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};
use std::{cell::RefCell, rc::Rc};

const DATE_FORMAT_STR: &str = "%H:%M:%S.%3f";

pub(crate) struct TimeStripTab {
    state: ListState,

    // shared context
    time_strip: Rc<RefCell<TimeStrip>>,
    pub(crate) selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl Widget for &mut TimeStripTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let time_stamps = &self.time_strip.borrow().time_stamps;
        let items: Vec<ListItem> = time_stamps
            .iter()
            .rev()
            .map(|ts| ListItem::from(MyTimeStamp(ts)))
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

    #[allow(clippy::await_holding_refcell_ref)]
    pub(crate) async fn handle_key_event(&mut self, event: KeyEvent) {
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
            KeyCode::Delete => {
                // delete the selected time stamp
                if let Some(time_stamp) = self.selected_time_stamp.borrow_mut().take() {
                    self.time_strip.borrow_mut().delete(&time_stamp).await.unwrap();
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

impl<'a> From<MyTimeStamp<'a>> for ListItem<'a> {
    fn from(value: MyTimeStamp<'a>) -> Self {
        let prefix: String = (value.0.split()).into();
        ListItem::new(format!(
            "{:5}  {}  {:3}  {:2}  {}",
            prefix,
            value.0.time.format(DATE_FORMAT_STR),
            value.0.heat_nr().unwrap_or_default(),
            value.0.bib_opt().unwrap_or_default(),
            match value.0.is_persisted() {
                true => "\u{1F506}",
                false => "\u{1F329}",
            }
        ))
    }
}

struct MyTimeStamp<'a>(&'a TimeStamp);
