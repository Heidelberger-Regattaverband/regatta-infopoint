mod heats_tab;
mod logs_tab;
mod selected_tab;
mod timestrip_popup;
mod timestrip_tab;
mod utils;

use crate::{
    app::{selected_tab::SelectedTab, timestrip_popup::TimeStripTabPopup, timestrip_tab::TimeStripTab},
    aquarius::{
        client::Client,
        messages::{EventHeatChanged, Heat},
    },
    args::Args,
    error::TimekeeperErr,
};
use clap::Parser;
use db::{tiberius::TiberiusPool, timekeeper::TimeStrip};
use heats_tab::HeatsTab;
use log::{debug, warn};
use logs_tab::LogsTab;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{
        Constraint::{self, Length, Min},
        Flex, Layout, Rect,
    },
    style::Stylize,
    text::Line,
    widgets::{Clear, Tabs},
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
use strum::IntoEnumIterator;
use tiberius::{AuthMethod, Config, EncryptionLevel};

pub struct App<'a> {
    // application state
    state: AppState,
    selected_tab: SelectedTab,

    // event receiver
    receiver: Receiver<AppEvent>,

    // UI components
    heats_tab: HeatsTab,
    time_strip_tab: TimeStripTab,
    time_strip_popup: TimeStripTabPopup<'a>,
    logs_tab: LogsTab,

    // shared context
    client: Rc<RefCell<Client>>,
    heats: Rc<RefCell<Vec<Heat>>>,
    time_strip: Rc<RefCell<TimeStrip>>,
    show_time_strip_popup: Rc<RefCell<bool>>,
}

impl App<'_> {
    pub(crate) async fn new() -> Self {
        let args = Args::parse();

        let mut config = Config::new();
        config.host(&args.db_host);
        config.port(args.db_port);
        config.database(&args.db_name);
        config.authentication(AuthMethod::sql_server(&args.db_user, &args.db_password));
        config.encryption(EncryptionLevel::NotSupported);

        TiberiusPool::init(config, 1, 1).await;
        let timestrip = TimeStrip::load(TiberiusPool::instance()).await.unwrap();

        // Use an mpsc::channel to combine stdin events with app events
        let (sender, receiver) = mpsc::channel();

        let client: Client = Client::new(&args.host, args.port, args.timeout, sender.clone());
        thread::spawn(move || input_thread(sender.clone()));

        // shared context
        let client_rc = Rc::new(RefCell::new(client));
        let heats = Rc::new(RefCell::new(Vec::new()));
        let time_strip = Rc::new(RefCell::new(timestrip));
        let selected_time_stamp = Rc::new(RefCell::new(None));
        let show_time_strip_popup = Rc::new(RefCell::new(false));

        Self {
            state: AppState::Running,
            selected_tab: SelectedTab::Heats,
            // tabs
            heats_tab: HeatsTab::new(heats.clone()),
            time_strip_tab: TimeStripTab::new(
                time_strip.clone(),
                selected_time_stamp.clone(),
                show_time_strip_popup.clone(),
            ),
            time_strip_popup: TimeStripTabPopup::new(
                client_rc.clone(),
                heats.clone(),
                time_strip.clone(),
                selected_time_stamp.clone(),
                show_time_strip_popup.clone(),
            ),
            logs_tab: LogsTab::default(),
            // shared context
            client: client_rc,
            receiver,
            heats,
            time_strip,
            show_time_strip_popup,
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
                        if *self.show_time_strip_popup.borrow() {
                            let popup_area = popup_area(inner_area, 50, 20);
                            frame.render_widget(Clear, popup_area); // this clears out the background
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
            self.client.borrow_mut().disconnect();
            self.heats.borrow_mut().clear();
        } else {
            let _ = self.client.borrow_mut().connect();
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
                        KeyCode::Char('q') => self.state = AppState::Quitting,
                        KeyCode::Char('+') => self.time_strip.borrow_mut().add_new_start(),
                        KeyCode::Char(' ') => self.time_strip.borrow_mut().add_new_finish(),
                        KeyCode::Char('r') => self.read_open_heats(),
                        _ => match self.selected_tab {
                            SelectedTab::Heats => self.heats_tab.handle_key_event(key_event),
                            SelectedTab::TimeStrip => {
                                if *self.show_time_strip_popup.borrow() {
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
                debug!("Mouse event: {mouse:?}");
            }
            _ => {}
        }
    }

    fn handle_aquarius_event(&mut self, event: EventHeatChanged) {
        let mut heats = self.heats.borrow_mut();
        match event.opened {
            true => {
                heats.push(event.heat);
                heats.sort_by(|a, b| a.number.cmp(&b.number));
            }
            false => {
                let index = heats.iter().position(|heat| heat.id == event.heat.id);
                if let Some(index) = index {
                    heats.remove(index);
                }
            }
        }
    }

    fn read_open_heats(&mut self) {
        match self.client.borrow_mut().read_open_heats() {
            Ok(open_heats) => {
                let mut heats = self.heats.borrow_mut();
                open_heats.iter().for_each(|heat| {
                    if !heats.contains(heat) {
                        heats.push(heat.clone());
                    }
                });
                heats.sort_by(|a, b| a.number.cmp(&b.number));
            }
            Err(err) => warn!("Error reading open heats: {err}"),
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
    while let Ok(event) = event::read() {
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
