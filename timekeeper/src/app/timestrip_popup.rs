use crate::app::utils::popup_block;
use crossterm::event::KeyEvent;
use log::warn;
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use tui_textarea::{Input, TextArea};

#[derive(Default)]
pub(crate) struct TimeStripTabPopup<'a> {
    input: TextArea<'a>,
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
        warn!("Input {:?}", input);
        self.input.input(input);
    }
}
