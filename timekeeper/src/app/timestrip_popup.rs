use crate::aquarius::{client::Client, messages::Heat};
use db::timekeeper::{Split, TimeStamp, TimeStrip};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::StatefulWidget;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{Block, BorderType, Padding, Widget},
};
use std::{cell::RefCell, rc::Rc};
use tui_prompts::prelude::*;

pub(crate) struct TimeStripTabPopup<'a> {
    heat_state: TextState<'a>,
    bib_state: TextState<'a>,
    current_field: Field,

    // shared context
    client: Rc<RefCell<Client>>,
    heats: Rc<RefCell<Vec<Heat>>>,
    time_strip: Rc<RefCell<TimeStrip>>,
    selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl Widget for &mut TimeStripTabPopup<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let binding = self.selected_time_stamp.borrow_mut();
        let ts_split = binding.as_ref().unwrap().split();
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1))
            .title(format!(" {ts_split} "));

        // inner popup area
        let [heat_area, bib_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(block.inner(area));
        block.render(area, buffer);

        TextPrompt::from("Lauf    ").render(heat_area, buffer, &mut self.heat_state);
        if matches!(ts_split, Split::Finish) {
            TextPrompt::from("Startnr.").render(bib_area, buffer, &mut self.bib_state);
        }
    }
}

impl<'a> TimeStripTabPopup<'a> {
    pub(crate) fn new(
        client: Rc<RefCell<Client>>,
        heats: Rc<RefCell<Vec<Heat>>>,
        time_strip: Rc<RefCell<TimeStrip>>,
        selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
        show_time_strip_popup: Rc<RefCell<bool>>,
    ) -> Self {
        Self {
            heat_state: TextState::default(),
            bib_state: TextState::default(),
            current_field: Field::default(),
            client,
            heats,
            time_strip,
            selected_time_stamp,
            show_time_strip_popup,
        }
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub(crate) async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, _) => {
                // close popup without saving
                *self.show_time_strip_popup.borrow_mut() = false;
            }
            (KeyCode::Enter, _) => {
                self.submit();
                // if input is finished close popup
                *self.show_time_strip_popup.borrow_mut() = !self.is_finished();
                // if self.is_valid {
                //     let heat_nr = self.heat_state.value().parse::<i16>().unwrap();
                //     self.heat_state.delete();
                //     if let Some(time_stamp) = self.selected_time_stamp.borrow().as_ref()
                //         && let Ok(time_stamp) = self.time_strip.borrow_mut().set_heat_nr(time_stamp, heat_nr).await
                //     {
                //         *self.show_time_strip_popup.borrow_mut() = false;
                //         self.client.borrow_mut().send_time(&time_stamp, None).unwrap();
                //     }
                //     self.is_valid = false;
                // }
            }
            _ => {
                self.current_state().handle_key_event(key_event);
                // self.validate();
            }
        }
    }

    fn validate(&mut self) -> bool {
        match self.current_field {
            Field::Heat => self.validate_heat(),
            Field::Bib => self.validate_bib(),
        }
        // if self.is_valid {
        //     self.heat_state.sta set_style(Style::default().fg(Color::LightGreen));
        // } else {
        //     self.heat_state.set_style(Style::default().fg(Color::LightRed));
        // }
    }

    fn validate_heat(&self) -> bool {
        if let Ok(heat_nr) = self.heat_state.value().parse::<i16>() {
            self.heats.borrow().iter().any(|heat| heat.number == heat_nr)
        } else {
            false
        }
    }
    fn validate_bib(&self) -> bool {
        if let (Ok(heat_nr), Ok(bib_nr)) = (
            self.heat_state.value().parse::<i16>(),
            self.bib_state.value().parse::<u8>(),
        ) {
            self.heats.borrow().iter().any(|heat| {
                heat.number == heat_nr && heat.boats.as_ref().unwrap().iter().any(|boat| boat.bib == bib_nr)
            })
        } else {
            false
        }
    }

    fn submit(&mut self) {
        if self.validate() {
            self.current_state().complete();
            if self.current_state().is_finished() && !self.is_finished() {
                self.focus_next();
            }
        }
    }
    fn focus_next(&mut self) {
        self.current_state().blur();
        self.current_field = self.next_field();
        self.current_state().focus();
    }
    const fn next_field(&self) -> Field {
        match self.current_field {
            Field::Heat => Field::Bib,
            Field::Bib => Field::Heat,
        }
    }
    const fn current_state(&mut self) -> &mut TextState<'a> {
        match self.current_field {
            Field::Heat => &mut self.heat_state,
            Field::Bib => &mut self.bib_state,
        }
    }
    const fn is_finished(&self) -> bool {
        self.heat_state.is_finished() && self.bib_state.is_finished()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum Field {
    #[default]
    Heat,
    Bib,
}
