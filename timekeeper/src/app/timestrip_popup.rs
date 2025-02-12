use crate::app::utils::popup_block;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use tui_textarea::{Input, TextArea};

#[derive(Default)]
pub(crate) struct TimeStripTabPopup<'a> {
    input: TextArea<'a>,
    pub(crate) heats: Vec<u16>,
    is_valid: bool,
}

impl Widget for &mut TimeStripTabPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.input.set_block(popup_block().title(" Lauf "));
        self.input.set_cursor_line_style(Style::default());
        self.input.render(area, buf);
    }
}

impl TimeStripTabPopup<'_> {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        let input: Input = event.into();
        if self.input.input(input) {
            self.validate();
        }
    }
    fn validate(&mut self) {
        if let Ok(heat_nr) = self.input.lines()[0].parse::<u16>() {
            self.is_valid = self.heats.contains(&heat_nr);
        } else {
            self.is_valid = false;
        }
        if self.is_valid {
            self.input.set_style(Style::default().fg(Color::LightGreen));
        } else {
            self.input.set_style(Style::default().fg(Color::LightRed));
        }
    }
}
