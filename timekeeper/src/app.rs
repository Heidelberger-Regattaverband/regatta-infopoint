use crate::{
    args::Args,
    client::{Client, HeatEventReceiver},
    error::MessageErr,
    messages::EventHeatChanged,
};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use log::{debug, info};
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint::{self, Length, Min},
        Layout, Rect,
    },
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
    DefaultTerminal,
};
use std::io::Result as IoResult;
use std::sync::{Arc, Mutex};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);
        self.selected_tab.render(inner_area, buf);
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

impl App {
    /// runs the application's main loop until the user quits
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), MessageErr> {
        let args = Args::parse();

        let mut client = Client::connect(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
        let open_heats = client.read_open_heats()?;
        debug!("Open heats: {:#?}", open_heats);

        let receiver = Arc::new(Mutex::new(EventReceiver));

        client.start_receiving_events(receiver).map_err(MessageErr::IoError)?;

        while self.state == AppState::Running {
            terminal
                .draw(|frame| frame.render_widget(&*self, frame.area()))
                .map_err(MessageErr::IoError)?;
            self.handle_events().map_err(MessageErr::IoError)?;
        }
        Ok(())
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles).select(selected_tab_index).render(area, buf);
    }

    fn handle_events(&mut self) -> IoResult<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
                    KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    _ => {}
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "Zeitmessung")]
    Tab1,
    #[strum(to_string = "Zeitstreifen")]
    Tab2,
    #[strum(to_string = "Logs")]
    Tab3,
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // in a real app these might be separate widgets
        match self {
            Self::Tab1 => self.render_tab0(area, buf),
            Self::Tab2 => self.render_tab1(area, buf),
            Self::Tab3 => self.render_tab2(area, buf),
        }
    }
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

    fn render_tab0(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Hello, World!").block(self.block()).render(area, buf);
    }

    fn render_tab1(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Welcome to the Ratatui tabs example!")
            .block(self.block())
            .render(area, buf);
    }

    fn render_tab2(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Look! I'm different than others!")
            .block(self.block())
            .render(area, buf);
    }

    /// A block surrounding the tab's content
    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
    }
}
