use crate::app::utils::popup_block;
use crossterm::event::{KeyCode, KeyEvent};
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
    pub(crate) is_valid: bool,
    pub(crate) selected_heat: Option<u16>,
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
        match event.code {
            KeyCode::Esc => {
                self.input.delete_line_by_head();
            }
            KeyCode::Enter => {
                if self.is_valid {
                    let heat_nr = self.input.lines()[0].parse::<u16>().unwrap();
                    self.selected_heat = Some(heat_nr);
                    self.input.delete_line_by_head();
                }
            }
            _ => {
                let input: Input = event.into();
                if self.input.input(input) {
                    self.validate();
                }
            }
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
