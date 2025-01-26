mod tabs;

use crate::{
    aquarius::{
        client::{Client, HeatEventReceiver},
        messages::EventHeatChanged,
    },
    args::Args,
    error::MessageErr,
    timestrip::TimeStrip,
};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use log::{debug, info, trace};
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint::{self, Length, Min},
        Layout, Rect,
    },
    style::Stylize,
    text::Line,
    widgets::{Tabs, Widget},
    DefaultTerminal,
};
use std::{
    io::Result as IoResult,
    sync::mpsc::{self, Sender},
};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tabs::{logs::LogsTab, measurement::TimeMeasurementTab, timestrip::TimeStripTab};

struct EventReceiver;

impl HeatEventReceiver for EventReceiver {
    fn on_event(&mut self, event: &EventHeatChanged) {
        info!("Received event: {:?}", &event);
    }
}

#[derive(Default)]
pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    measurement_tab: TimeMeasurementTab,
    time_strip_tab: TimeStripTab,
    logs_tab: LogsTab,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);
        self.render_selected_tab(inner_area, buf);
        render_footer(footer_area, buf);
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    "Aquarius Zeitmessung".bold().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::raw("◄ ► to change tab | Press q to quit")
        .centered()
        .render(area, buf);
}

fn input_thread(tx_event: Sender<AppEvent>) -> Result<(), MessageErr> {
    trace!(target:"crossterm", "Starting input thread");
    while let Ok(event) = event::read() {
        trace!(target:"crossterm", "Stdin event received {:?}", event);
        tx_event.send(AppEvent::UiEvent(event)).map_err(MessageErr::SendError)?;
    }
    Ok(())
}

impl App {
    pub(crate) fn start(mut self, terminal: &mut DefaultTerminal) -> Result<(), MessageErr> {
        // Use an mpsc::channel to combine stdin events with app events
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();

        thread::spawn(move || input_thread(event_tx));

        self.run(terminal, rx)
    }

    /// runs the application's main loop until the user quits
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<AppEvent>) -> Result<(), MessageErr> {
        let args = Args::parse();

        let mut client = Client::connect(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
        let open_heats = client.read_open_heats()?;
        debug!("Open heats: {:#?}", open_heats);

        let receiver = Arc::new(Mutex::new(EventReceiver));

        client.start_receiving_events(receiver).map_err(MessageErr::IoError)?;

        // main loop, runs until the user quits the application by pressing 'q'
        for event in rx {
            match event {
                AppEvent::UiEvent(event) => self.handle_ui_event(event, &mut client).map_err(MessageErr::IoError)?,
            }
            if self.state == AppState::Quitting {
                break;
            }
            terminal
                .draw(|frame| frame.render_widget(&*self, frame.area()))
                .map_err(MessageErr::IoError)?;
        }
        Ok(())
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles).select(selected_tab_index).render(area, buf);
    }

    fn render_selected_tab(&self, area: Rect, buf: &mut Buffer) {
        match self.selected_tab {
            SelectedTab::Measurement => self.measurement_tab.render(area, buf),
            SelectedTab::TimeStrip => self.time_strip_tab.render(area, buf),
            SelectedTab::Logs => self.logs_tab.render(area, buf),
        };
    }

    fn handle_ui_event(&mut self, event: Event, client: &mut Client) -> IoResult<()> {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Right => self.next_tab(),
                    KeyCode::Left => self.previous_tab(),
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    KeyCode::Char(' ') => self.time_strip_tab.finish_time_stamp(),
                    KeyCode::Enter => self.time_strip_tab.start_time_stamp(),
                    KeyCode::Char('r') => match client.read_open_heats() {
                        Ok(open_heats) => {
                            debug!("Open heats: {:#?}", open_heats);
                        }
                        Err(err) => {
                            debug!("Error reading open heats: {}", err);
                        }
                    },
                    code => {
                        debug!("Unhandled key code: {:?}", code);
                    }
                }
            }
        }
        Ok(())
    }

    fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

/// The application's state (running or quitting)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    /// The application is running
    #[default]
    Running,
    /// The application is quitting
    Quitting,
}

#[derive(Debug)]
pub(crate) enum AppEvent {
    /// An UI event
    UiEvent(Event),
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Zeitmessung")]
    Measurement,
    #[strum(to_string = "Zeitstreifen")]
    TimeStrip,
    #[strum(to_string = "Logs")]
    Logs,
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    /// Return tab's name as a styled `Line`
    fn title(self) -> Line<'static> {
        format!("  {self}  ").into()
    }
}
