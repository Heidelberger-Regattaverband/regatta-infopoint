use crate::aquarius::{client::Client, messages::Heat};
use db::timekeeper::{TimeStamp, TimeStrip};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};
use std::{cell::RefCell, rc::Rc};
use tui_prompts::prelude::*;

pub(crate) struct TimeStripTabPopup<'a> {
    heat_state: TextState<'a>,
    bib_state: TextState<'a>,
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
        let ts_split = binding.as_ref().unwrap().split();
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1))
            .title(format!(" {ts_split} "));

        // inner popup area
        let inner_area = block.inner(area);
        block.render(area, buf);

        let label_txt = "Lauf #:";
        // horizontal header layout: tabs, title
        let [label_area, input_area] = Layout::horizontal([
            Constraint::Length((label_txt.len() + 2).try_into().unwrap()),
            Constraint::Fill(1),
        ])
        .areas(inner_area);
        Paragraph::new(label_txt).render(label_area, buf);
        TextPrompt::from("Lauf #")
            .with_block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1)),
            )
            .draw(input_area, buf);
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
            heat_state: TextState::default(),
            bib_state: TextState::default(),
            is_valid: false,
            client,
            heats,
            time_strip,
            selected_time_stamp,
            show_time_strip_popup,
        }
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub(crate) async fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => {
                if self.heat_state.is_empty() {
                    *self.show_time_strip_popup.borrow_mut() = false;
                } else {
                    self.heat_state.delete();
                }
            }
            KeyCode::Enter => {
                if self.is_valid {
                    let heat_nr = self.heat_state.value().parse::<i16>().unwrap();
                    self.heat_state.delete();
                    if let Some(time_stamp) = self.selected_time_stamp.borrow().as_ref()
                        && let Ok(time_stamp) = self.time_strip.borrow_mut().set_heat_nr(time_stamp, heat_nr).await
                    {
                        *self.show_time_strip_popup.borrow_mut() = false;
                        self.client.borrow_mut().send_time(&time_stamp, None).unwrap();
                    }
                    self.is_valid = false;
                }
            }
            _ => {
                // let input: Input = event.into();
                // if self.heat_state.input(input) {
                //     self.validate();
                // }
            }
        }
    }

    fn validate(&mut self) {
        if let Ok(heat_nr) = self.heat_state.value().parse::<i16>() {
            self.is_valid = self.heats.borrow().iter().any(|heat| heat.number == heat_nr);
        } else {
            self.is_valid = false;
        }
        // if self.is_valid {
        //     self.heat_state.set_style(Style::default().fg(Color::LightGreen));
        // } else {
        //     self.heat_state.set_style(Style::default().fg(Color::LightRed));
        // }
    }
}
