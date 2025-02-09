mod tabs;

use crate::{
    aquarius::{client::Client, messages::EventHeatChanged},
    args::Args,
    error::TimekeeperErr,
    timestrip::TimeStrip,
};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use log::{debug, info, trace, warn};
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
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use strum::IntoEnumIterator;
use tabs::{logs::LogsTab, measurement::TimeMeasurementTab, timestrip::TimeStripTab, SelectedTab};

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    measurement_tab: TimeMeasurementTab,
    time_strip_tab: TimeStripTab,
    logs_tab: LogsTab,
    client: Client,
    receiver: Receiver<AppEvent>,
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // vertical layout: header, inner area, footer
        let [header_area, inner_area, footer_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)]).areas(area);

        // horizontal header layout: tabs, title
        let [tabs_area, title_area] = Layout::horizontal([Min(0), Length(20)]).areas(header_area);

        // render tabs header and title
        "Aquarius Zeitmessung".bold().render(title_area, buf);
        let titles = SelectedTab::iter().map(SelectedTab::title);

        // render the selected tab
        Tabs::new(titles)
            .select(self.selected_tab as usize)
            .render(tabs_area, buf);
        match self.selected_tab {
            SelectedTab::Measurement => self.measurement_tab.render(inner_area, buf),
            SelectedTab::TimeStrip => self.time_strip_tab.render(inner_area, buf),
            SelectedTab::Logs => self.logs_tab.render(inner_area, buf),
        };

        // render footer
        Line::raw("◄ ► to change tab | Press q to quit")
            .centered()
            .render(footer_area, buf);
    }
}

impl App {
    pub(crate) fn new() -> Self {
        // Use an mpsc::channel to combine stdin events with app events
        let (sender, receiver) = mpsc::channel();

        let args = Args::parse();
        let client = Client::new(&args.host, args.port, args.timeout, sender.clone());
        thread::spawn(move || input_thread(sender.clone()));

        Self {
            state: AppState::Running,
            selected_tab: SelectedTab::Measurement,
            measurement_tab: TimeMeasurementTab::default(),
            time_strip_tab: TimeStripTab::default(),
            logs_tab: LogsTab::default(),
            client,
            receiver,
        }
    }

    pub(crate) fn start(mut self, terminal: &mut DefaultTerminal) -> Result<(), TimekeeperErr> {
        // main loop, runs until the user quits the application by pressing 'q'
        while self.state == AppState::Running {
            let event = self.receiver.recv().map_err(TimekeeperErr::ReceiveError)?;
            match event {
                AppEvent::UI(event) => self.handle_ui_event(event),
                AppEvent::Aquarius(event) => self.handle_aquarius_event(event),
                AppEvent::Client(connected) => self.handle_client_event(connected),
            }
            self.draw(terminal)?;
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut DefaultTerminal) -> Result<(), TimekeeperErr> {
        terminal
            .draw(|frame| frame.render_widget(self, frame.area()))
            .map_err(TimekeeperErr::IoError)?;
        Ok(())
    }

    fn handle_client_event(&mut self, connected: bool) {
        if !connected {
            self.client.disconnect();
            warn!("Connection lost, reconnecting...");
        } else {
            let _ = self.client.connect();
            info!("Connection established.");
        }
    }

    fn handle_ui_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Right => self.next_tab(),
                        KeyCode::Left => self.previous_tab(),
                        KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                        KeyCode::Char('r') => match self.client.read_open_heats() {
                            Ok(open_heats) => {
                                debug!("Open heats: {:?}", open_heats);
                            }
                            Err(err) => {
                                debug!("Error reading open heats: {}", err);
                            }
                        },
                        _ => self.time_strip_tab.handle_key_event(key_event),
                    }
                }
            }
            Event::Mouse(mouse) => {
                debug!("Mouse event: {:?}", mouse);
            }
            _ => {}
        }
    }

    fn handle_aquarius_event(&mut self, event: EventHeatChanged) {
        debug!("Received event: {:?}", event);
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

fn input_thread(sender: Sender<AppEvent>) -> Result<(), TimekeeperErr> {
    trace!(target:"crossterm", "Starting input thread");
    while let Ok(event) = event::read() {
        trace!(target:"crossterm", "Stdin event received {:?}", event);
        sender.send(AppEvent::UI(event)).map_err(TimekeeperErr::SendError)?;
    }
    Ok(())
}

/// The application's state (running or quitting)
#[derive(Default, PartialEq, Eq)]
enum AppState {
    /// The application is running
    #[default]
    Running,
    /// The application is quitting
    Quitting,
}

pub(crate) enum AppEvent {
    /// An UI event
    UI(Event),

    /// An event from Aquarius
    Aquarius(EventHeatChanged),

    /// An event from the client, e.g. connection lost
    Client(bool),
}
