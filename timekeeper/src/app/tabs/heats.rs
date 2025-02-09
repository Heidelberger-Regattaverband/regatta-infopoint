use crate::{
    app::tabs::block,
    aquarius::messages::{EventHeatChanged, Heat},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};

#[derive(Default)]
pub(crate) struct HeatsTab {
    heats: Vec<Heat>,
    state: ListState,
}

impl Widget for &mut HeatsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.heats.iter().map(ListItem::from).collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block())
            .highlight_symbol(">>  ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

impl HeatsTab {
    pub(crate) fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => self.state.select_next(),
            KeyCode::Home => self.state.select_first(),
            KeyCode::End => self.state.select_last(),
            KeyCode::Char('h') => self.state.select(None),
            _ => {}
        }
    }

    pub(crate) fn handle_aquarius_event(&mut self, event: EventHeatChanged) {
        match event.opened {
            true => {
                self.heats.push(event.heat);
                self.heats.sort_by(|a, b| a.number.cmp(&b.number));
            }
            false => {
                let index = self.heats.iter().position(|heat| heat.id == event.heat.id);
                if let Some(index) = index {
                    self.heats.remove(index);
                }
            }
        }
    }

    pub(crate) fn set_heats(&mut self, heats: Vec<Heat>) {
        self.heats = heats;
        self.heats.sort_by(|a, b| a.number.cmp(&b.number));
    }

    pub(crate) fn clear_heats(&mut self) {
        self.heats.clear();
    }
}

impl From<&Heat> for ListItem<'_> {
    fn from(heat: &Heat) -> Self {
        let boats = heat
            .boats
            .clone()
            .or_else(|| Some(Vec::new()))
            .unwrap()
            .iter()
            .map(|boat| format!("{:2}: {}", boat.bib, boat.club))
            .collect::<Vec<String>>()
            .join("\n       ");
        ListItem::new(format!("#{:3} - {}\n\n", heat.number, boats))
    }
}
