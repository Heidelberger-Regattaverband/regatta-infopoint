use crate::{
    app::utils::popup_block,
    aquarius::{client::Client, messages::Heat},
    timestrip::{TimeStamp, TimeStrip},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use std::{cell::RefCell, rc::Rc};
use tui_textarea::{Input, TextArea};

pub(crate) struct TimeStripTabPopup<'a> {
    input: TextArea<'a>,
    is_valid: bool,

    // shared context
    client: Rc<RefCell<Client>>,
    heats: Rc<RefCell<Vec<Heat>>>,
    time_strip: Rc<RefCell<TimeStrip>>,
    selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl Widget for &mut TimeStripTabPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let binding = self.selected_time_stamp.borrow_mut();
        let ts = binding.as_ref().unwrap();
        self.input
            .set_block(popup_block().title(format!(" {} #{} ", ts.stamp_type, ts.index)));
        self.input.render(area, buf);
    }
}

impl TimeStripTabPopup<'_> {
    pub(crate) fn new(
        client: Rc<RefCell<Client>>,
        heats: Rc<RefCell<Vec<Heat>>>,
        time_strip: Rc<RefCell<TimeStrip>>,
        selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
        show_time_strip_popup: Rc<RefCell<bool>>,
    ) -> Self {
        Self {
            input: TextArea::default(),
            is_valid: false,
            client,
            heats,
            time_strip,
            selected_time_stamp,
            show_time_strip_popup,
        }
    }

    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => {
                if self.input.is_empty() {
                    *self.show_time_strip_popup.borrow_mut() = false;
                } else {
                    self.input.delete_line_by_head();
                }
            }
            KeyCode::Enter => {
                if self.is_valid {
                    let heat_nr = self.input.lines()[0].parse::<u16>().unwrap();
                    self.input.delete_line_by_head();
                    if let Some(time_stamp) = self.selected_time_stamp.borrow().as_ref() {
                        self.time_strip.borrow_mut().assign_heat_nr(time_stamp.index, heat_nr);
                        *self.show_time_strip_popup.borrow_mut() = false;
                    }
                    self.is_valid = false;
                }
            }
            _ => {
                let input: Input = event.into();
                if self.input.input(input) {
                    self.validate();
                }
            }
        }
    }

    fn validate(&mut self) {
        if let Ok(heat_nr) = self.input.lines()[0].parse::<u16>() {
            self.is_valid = self.heats.borrow().iter().any(|heat| heat.number == heat_nr);
        } else {
            self.is_valid = false;
        }
        if self.is_valid {
            self.input.set_style(Style::default().fg(Color::LightGreen));
        } else {
            self.input.set_style(Style::default().fg(Color::LightRed));
        }
    }
}
