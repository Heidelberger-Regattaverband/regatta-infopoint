use crate::app::tabs::block;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
#[derive(Default)]
pub(crate) struct LogsTab {}

impl Widget for &mut LogsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TuiLoggerWidget::default()
            .block(block())
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(true)
            .output_file(false)
            .output_line(false)
            .render(area, buf);
    }
}
