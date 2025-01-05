use crate::{args::Args, client::Client, error::MessageErr, messages::EventHeatChanged, utils};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use log::{debug, info, warn};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::thread;

#[derive(Debug)]
pub struct App {
    counter: u8,
    exit: bool,
    thread: Option<thread::JoinHandle<()>>,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            counter: 0,
            exit: false,
            thread: None,
        }
    }

    /// runs the application's main loop until the user quits
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), MessageErr> {
        let args = Args::parse();

        let mut client = Client::new(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
        let open_heats = client.read_open_heats()?;
        debug!("Open heats: {:#?}", open_heats);

        info!("Spawning thread to receive events.");
        self.thread = Some(thread::spawn(move || loop {
            info!("Receiving events ...");
            let received = client.receive_line().unwrap();
            if !received.is_empty() {
                debug!("Received: \"{}\"", utils::print_whitespaces(&received));
                let event_opt = EventHeatChanged::parse(&received);
                match event_opt {
                    Ok(mut event) => {
                        if event.opened {
                            client.read_start_list(&mut event.heat).unwrap();
                        }
                    }
                    Err(err) => handle_error(err),
                }
            }
        }));

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

fn handle_error(err: MessageErr) {
    match err {
        MessageErr::ParseError(parse_err) => {
            warn!("Error parsing number: {}", parse_err);
        }
        MessageErr::IoError(io_err) => {
            warn!("I/O error: {}", io_err);
        }
        MessageErr::InvalidMessage(message) => {
            warn!("Invalid message: {}", message);
        }
    }
}
