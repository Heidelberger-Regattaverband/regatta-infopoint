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
    pub(crate) is_valid: bool,
    pub(crate) selected_heat: Option<u16>,

    // shared context
    client: Rc<RefCell<Client>>,
    heats: Rc<RefCell<Vec<Heat>>>,
    time_strip: Rc<RefCell<TimeStrip>>,
    selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
}

impl Widget for &mut TimeStripTabPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.input.set_block(popup_block().title(" Lauf "));
        // self.input.set_cursor_line_style(Style::default());
        self.input.render(area, buf);
    }
}

impl TimeStripTabPopup<'_> {
    pub(crate) fn new(
        client: Rc<RefCell<Client>>,
        heats: Rc<RefCell<Vec<Heat>>>,
        time_strip: Rc<RefCell<TimeStrip>>,
        selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    ) -> Self {
        Self {
            input: TextArea::default(),
            is_valid: false,
            selected_heat: None,
            client,
            heats,
            time_strip,
            selected_time_stamp,
        }
    }

    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => {
                self.input.delete_line_by_head();
            }
            KeyCode::Enter => {
                if self.is_valid {
                    let heat_nr = self.input.lines()[0].parse::<u16>().unwrap();
                    self.selected_heat = Some(heat_nr);
                    self.input.delete_line_by_head();
                    // self.client.borrow_mut().send_heat_change(heat_nr);
                    if let Some(time_stamp) = self.selected_time_stamp.borrow().as_ref() {
                        self.time_strip.borrow_mut().assign_heat_nr(time_stamp.index, heat_nr);
                    }
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
