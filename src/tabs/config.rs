
use ratatui::{prelude::*, widgets::*};
use crate::theme::THEME;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigTab {
    row_index: usize,
}

impl ConfigTab {
    pub fn enter_key(&mut self) {}
    pub fn prev_row(&mut self) {
        self.row_index = self.row_index.saturating_sub(1);
    }

    pub fn next_row(&mut self) {
        self.row_index = self.row_index.saturating_add(1);
    }
}
impl Widget for ConfigTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // herein lies the ui code for the tab
        Paragraph::new("CONFIG GOES HERE")
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Nodes")
                    .title_alignment(Alignment::Center)
                    .border_set(symbols::border::DOUBLE)
                    .style(THEME.middle)
            ).render(area, buf);
    }
}