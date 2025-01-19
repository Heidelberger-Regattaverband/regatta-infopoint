use super::block;
use crate::app::TimeStrip;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

#[derive(Default)]
pub(crate) struct TimeStripTab {
    // Add fields here
    time_strip: TimeStrip,
}

impl Widget for &TimeStripTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Welcome to the Ratatui tabs example!")
            .block(block())
            .render(area, buf);
    }
}

impl TimeStripTab {
    pub(crate) fn finish_time_stamp(&mut self) {
        self.time_strip.add_new_finish();
    }

    pub(crate) fn start_time_stamp(&mut self) {
        self.time_strip.add_new_start();
    }
}
