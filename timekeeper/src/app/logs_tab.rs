use crate::app::utils::block;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::Widget,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

#[derive(Default)]
pub(crate) struct LogsTab {
    state: TuiWidgetState,
}

impl LogsTab {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        let tui_event = match event.code {
            KeyCode::Up => TuiWidgetEvent::UpKey,
            KeyCode::Down => TuiWidgetEvent::DownKey,
            KeyCode::Left => TuiWidgetEvent::LeftKey,
            KeyCode::Right => TuiWidgetEvent::RightKey,
            KeyCode::PageUp => TuiWidgetEvent::PrevPageKey,
            KeyCode::PageDown => TuiWidgetEvent::NextPageKey,
            KeyCode::Esc => TuiWidgetEvent::EscapeKey,
            KeyCode::Char(' ') => TuiWidgetEvent::SpaceKey,
            KeyCode::Char('+') => TuiWidgetEvent::PlusKey,
            KeyCode::Char('-') => TuiWidgetEvent::MinusKey,
            KeyCode::Char('h') => TuiWidgetEvent::HideKey,
            _ => return,
        };

        self.state.transition(tui_event);
    }
}

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
            .state(&self.state)
            .render(area, buf);
    }
}
