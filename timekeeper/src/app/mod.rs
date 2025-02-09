mod tabs;

use crate::{
    aquarius::{client::Client, messages::EventHeatChanged},
    args::Args,
    error::TimekeeperErr,
    timestrip::TimeStrip,
};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use log::{debug, trace, warn};
use ratatui::{
    layout::{
        Constraint::{self, Length, Min},
        Flex, Layout, Rect,
    },
    style::Stylize,
    text::Line,
    widgets::{Clear, Tabs},
    DefaultTerminal,
};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use strum::IntoEnumIterator;
use tabs::{
    heats::HeatsTab,
    logs::LogsTab,
    timestrip::{TimeStripTab, TimeStripTabPopup},
    SelectedTab,
};

pub struct App {
    state: AppState,
    selected_tab: SelectedTab,
    heats_tab: HeatsTab,
    time_strip_tab: TimeStripTab,
    time_strip_popup: TimeStripTabPopup<'static>,
    logs_tab: LogsTab,
    client: Client,
    receiver: Receiver<AppEvent>,
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
            selected_tab: SelectedTab::Heats,
            heats_tab: HeatsTab::default(),
            time_strip_tab: TimeStripTab::default(),
            time_strip_popup: TimeStripTabPopup::default(),
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
            .draw(|frame| {
                // vertical layout: header, inner area, footer
                let [header_area, inner_area, footer_area] =
                    Layout::vertical([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
                        .areas(frame.area());
                // horizontal header layout: tabs, title
                let [tabs_area, title_area] = Layout::horizontal([Min(0), Length(20)]).areas(header_area);

                // render tabs header and title
                frame.render_widget("Aquarius Zeitmessung".bold(), title_area);
                let titles = SelectedTab::iter().map(SelectedTab::title);

                // render the selected tab
                frame.render_widget(Tabs::new(titles).select(self.selected_tab as usize), tabs_area);
                match self.selected_tab {
                    SelectedTab::Heats => frame.render_widget(&mut self.heats_tab, inner_area),
                    SelectedTab::TimeStrip => {
                        frame.render_widget(&mut self.time_strip_tab, inner_area);
                        if self.time_strip_tab.show_popup {
                            let popup_area = popup_area(inner_area, 50, 20);
                            frame.render_widget(Clear, popup_area); //this clears out the background
                            frame.render_widget(&mut self.time_strip_popup, popup_area);
                        }
                    }
                    SelectedTab::Logs => frame.render_widget(&mut self.logs_tab, inner_area),
                };

                // render footer
                frame.render_widget(Line::raw("◄ ► to change tab | Press q to quit").centered(), footer_area);
            })
            .map_err(TimekeeperErr::IoError)?;
        Ok(())
    }

    fn handle_client_event(&mut self, connected: bool) {
        if !connected {
            self.client.disconnect();
            self.heats_tab.clear_heats();
        } else {
            let _ = self.client.connect();
            self.read_open_heats();
        }
    }

    fn handle_ui_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Right => self.selected_tab = self.selected_tab.next(),
                        KeyCode::Left => self.selected_tab = self.selected_tab.previous(),
                        KeyCode::Char('q') | KeyCode::Esc => self.state = AppState::Quitting,
                        KeyCode::Char('+') => self.time_strip_tab.time_strip.add_new_start(),
                        KeyCode::Char(' ') => self.time_strip_tab.time_strip.add_new_finish(),
                        KeyCode::Char('r') => self.read_open_heats(),
                        _ => match self.selected_tab {
                            SelectedTab::Heats => self.heats_tab.handle_key_event(key_event),
                            SelectedTab::TimeStrip => {
                                if self.time_strip_tab.show_popup {
                                    self.time_strip_popup.handle_key_event(key_event);
                                } else {
                                    self.time_strip_tab.handle_key_event(key_event);
                                }
                            }
                            _ => {}
                        },
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
        self.heats_tab.handle_aquarius_event(event);
    }

    fn read_open_heats(&mut self) {
        match self.client.read_open_heats() {
            Ok(open_heats) => self.heats_tab.set_heats(open_heats),
            Err(err) => warn!("Error reading open heats: {}", err),
        };
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
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
