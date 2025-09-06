use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use tui_logger::{TuiLoggerSmartWidget, TuiWidgetEvent, TuiWidgetState};

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
            KeyCode::Char('f') => TuiWidgetEvent::FocusKey,
            _ => return,
        };

        self.state.transition(tui_event);
    }
}

impl Widget for &mut LogsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TuiLoggerSmartWidget::default()
            .border_style(Style::default().fg(Color::Black))
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::Magenta))
            .style_info(Style::default().fg(Color::Cyan))
            .state(&self.state)
            .render(area, buf);
    }
}
