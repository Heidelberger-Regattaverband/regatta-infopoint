use crate::app::tabs::block;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

#[derive(Default)]
pub(crate) struct TimeMeasurementTab {}

impl Widget for &TimeMeasurementTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Hello, World!").block(block()).render(area, buf);
    }
}
