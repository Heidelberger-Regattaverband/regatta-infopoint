use crate::{
    app::{
        TimeStrip,
        timestrip_popup::TimeStripTabPopup,
        utils::{HIGHLIGHT_SYMBOL, block},
    },
    aquarius::{client::Client, messages::Heat},
};
use db::timekeeper::TimeStamp;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Clear, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};
use std::{cell::RefCell, rc::Rc};

const DATE_FORMAT_STR: &str = "%H:%M:%S.%3f";

pub(crate) struct TimeStripTab<'a> {
    state: ListState,

    // UI components
    time_strip_popup: TimeStripTabPopup<'a>,

    // shared context
    client: Rc<RefCell<Client>>,
    heats: Rc<RefCell<Vec<Heat>>>,
    time_strip: Rc<RefCell<TimeStrip>>,
    selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl<'a> Widget for &mut TimeStripTab<'a> {
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

        if *self.show_time_strip_popup.borrow() {
            let popup_area = popup_area(area, 50, 20);
            Clear::default().render(popup_area, buf); // this clears out the background
            self.time_strip_popup.render(popup_area, buf);
        }
    }
}

impl<'a> TimeStripTab<'a> {
    pub(crate) fn new(
        client: Rc<RefCell<Client>>,
        heats: Rc<RefCell<Vec<Heat>>>,
        time_strip: Rc<RefCell<TimeStrip>>,
        selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    ) -> Self {
        let show_time_strip_popup = Rc::new(RefCell::new(false));
        Self {
            client: client.clone(),
            heats: heats.clone(),
            state: ListState::default(),
            time_strip: time_strip.clone(),
            selected_time_stamp: selected_time_stamp.clone(),
            show_time_strip_popup: show_time_strip_popup.clone(),
            time_strip_popup: TimeStripTabPopup::new(
                client,
                heats,
                time_strip,
                selected_time_stamp,
                show_time_strip_popup,
            ),
        }
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub(crate) async fn handle_key_event(&mut self, key_event: KeyEvent) {
        if *self.show_time_strip_popup.borrow() {
            self.time_strip_popup.handle_key_event(key_event).await;
        } else {
            match key_event.code {
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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
