use crate::{
    args::Args,
    client::{Client, HeatEventReceiver},
    error::MessageErr,
    messages::EventHeatChanged,
};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use log::{debug, info};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::sync::{Arc, Mutex};

struct EventReceiver;

impl HeatEventReceiver for EventReceiver {
    fn on_event(&mut self, event: &EventHeatChanged) {
        info!("Received event: {:?}", &event);
    }
}

#[derive(Debug)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            counter: 0,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), MessageErr> {
        let args = Args::parse();

        let mut client = Client::connect(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
        let open_heats = client.read_open_heats()?;
        debug!("Open heats: {:#?}", open_heats);

        let receiver = Arc::new(Mutex::new(EventReceiver));

        client.start_receiving_events(receiver).map_err(MessageErr::IoError)?;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame)).map_err(MessageErr::IoError)?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<(), MessageErr> {
        match event::read().map_err(MessageErr::IoError)? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self.handle_key_event(key_event),
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text).centered().block(block).render(area, buf);
    }
}
