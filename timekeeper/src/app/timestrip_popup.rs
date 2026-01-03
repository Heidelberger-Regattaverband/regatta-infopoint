use ::aquarius::{client::Client, messages::Heat};
use ::db::timekeeper::{TimeStamp, TimeStrip};
use ::ratatui::crossterm::event::Event;
use ::ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};
use ::std::{cell::RefCell, rc::Rc};
use ::tracing::info;
use ::tui_input::Input;
use ::tui_input::backend::crossterm::EventHandler;

pub(crate) struct TimeStripTabPopup {
    heat_input: Input,
    /// Current input mode
    heat_input_mode: InputMode,
    is_valid: bool,

    // shared context
    client: Rc<RefCell<Client>>,
    heats: Rc<RefCell<Vec<Heat>>>,
    time_strip: Rc<RefCell<TimeStrip>>,
    selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl Widget for &mut TimeStripTabPopup {
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
        info!(
            "TimeStripTabPopup rendering heat_input with value {}",
            self.heat_input.value()
        );
        Paragraph::new(self.heat_input.value()).render(input_area, buf);
    }
}

impl TimeStripTabPopup {
    pub(crate) fn new(
        client: Rc<RefCell<Client>>,
        heats: Rc<RefCell<Vec<Heat>>>,
        time_strip: Rc<RefCell<TimeStrip>>,
        selected_time_stamp: Rc<RefCell<Option<TimeStamp>>>,
        show_time_strip_popup: Rc<RefCell<bool>>,
    ) -> Self {
        Self {
            heat_input: Input::default(),
            heat_input_mode: InputMode::Normal,
            is_valid: false,
            client,
            heats,
            time_strip,
            selected_time_stamp,
            show_time_strip_popup,
        }
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub(crate) async fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            info!("TimeStripTabPopup handle key event: {:?}", key);
            match key.code {
                KeyCode::Esc => {
                    if self.heat_input.value().is_empty() {
                        *self.show_time_strip_popup.borrow_mut() = false;
                    } else {
                        self.heat_input.reset();
                    }
                }
                KeyCode::Enter => {
                    if self.is_valid {
                        let heat_nr = self.heat_input.value().parse::<i16>().unwrap();
                        self.heat_input.reset();
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
                    info!("TimeStripTabPopup passing event to heat_input");
                    if self.heat_input.handle_event(&event).is_some() {
                        info!("TimeStripTabPopup heat_input changed to {}", self.heat_input.value());
                        self.validate();
                    }
                }
            }
        }
    }

    pub(crate) fn set_heat_nr(&mut self, heat_nr: i16) {
        info!("TimeStripTabPopup set_heat_nr to {}", heat_nr);
        self.heat_input = Input::new(heat_nr.to_string());
        self.validate();
    }

    fn validate(&mut self) {
        if let Ok(heat_nr) = self.heat_input.value().parse::<i16>() {
            self.is_valid = self.heats.borrow().iter().any(|heat| heat.number == heat_nr);
        } else {
            self.is_valid = false;
        }
        // if self.is_valid {
        //     self.heat_input.set_style(Style::default().fg(Color::LightGreen));
        // } else {
        //     self.heat_input.set_style(Style::default().fg(Color::LightRed));
        // }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}
