mod tabs;

use crate::{
    aquarius::{client::Client, messages::EventHeatChanged},
    args::Args,
    error::TimekeeperErr,
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
use std::thread;
use std::{
    io::Result as IoResult,
    sync::mpsc::{self, Receiver, Sender},
};
use strum::IntoEnumIterator;
use tabs::{logs::LogsTab, measurement::TimeMeasurementTab, timestrip::TimeStripTab, SelectedTab};

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

        "Aquarius Zeitmessung".bold().render(title_area, buf);
        self.render_tabs(tabs_area, buf);
        self.render_selected_tab(inner_area, buf);
        Line::raw("◄ ► to change tab | Press q to quit")
            .centered()
            .render(footer_area, buf);
    }
}

fn input_thread(sender: Sender<AppEvent>) -> Result<(), TimekeeperErr> {
    trace!(target:"crossterm", "Starting input thread");
    while let Ok(event) = event::read() {
        trace!(target:"crossterm", "Stdin event received {:?}", event);
        sender
            .send(AppEvent::UiEvent(event))
            .map_err(TimekeeperErr::SendError)?;
    }
    Ok(())
}

impl App {
    pub(crate) fn start(mut self, terminal: &mut DefaultTerminal) -> Result<(), TimekeeperErr> {
        // Use an mpsc::channel to combine stdin events with app events
        let (sender, receiver) = mpsc::channel();
        let ui_event_sender = sender.clone();

        thread::spawn(move || input_thread(ui_event_sender));

        let args = Args::parse();
        let mut client = Client::new(args.host, args.port, args.timeout, sender.clone());
        client.connect().map_err(TimekeeperErr::IoError)?;
        let open_heats = client.read_open_heats()?;
        debug!("Open heats: {:#?}", open_heats);

        self.run(terminal, &mut client, receiver)
    }

    /// runs the application's main loop until the user quits
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        client: &mut Client,
        rx: Receiver<AppEvent>,
    ) -> Result<(), TimekeeperErr> {
        self.draw(terminal)?;

        // main loop, runs until the user quits the application by pressing 'q'
        for event in rx {
            match event {
                AppEvent::UiEvent(event) => self.handle_ui_event(event, client).map_err(TimekeeperErr::IoError)?,
                AppEvent::AquariusEvent(event) => {
                    info!("Received event: {:?}", &event);
                }
            }
            if self.state == AppState::Quitting {
                break;
            }
            self.draw(terminal)?;
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> Result<(), TimekeeperErr> {
        terminal
            .draw(|frame| frame.render_widget(&*self, frame.area()))
            .map_err(TimekeeperErr::IoError)?;
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

    /// An event from Aquarius
    AquariusEvent(EventHeatChanged),
}
