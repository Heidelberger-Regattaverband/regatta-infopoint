use crate::{
    app::utils::{block, HIGHLIGHT_SYMBOL},
    aquarius::messages::Heat,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget},
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct HeatsTab {
    heats: Rc<RefCell<Vec<Heat>>>,

    state: ListState,
}

impl Widget for &mut HeatsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.heats.borrow_mut().iter().map(ListItem::from).collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block())
            .highlight_symbol(HIGHLIGHT_SYMBOL)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

impl HeatsTab {
    pub(crate) fn new(heats: Rc<RefCell<Vec<Heat>>>) -> Self {
        Self {
            heats,
            state: ListState::default(),
        }
    }

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
