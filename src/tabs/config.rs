use crate::app::Mode;
use crate::theme::THEME;
use ratatui::{prelude::*, widgets::*};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigTab {
    row_index: usize,
}

impl ConfigTab {
    pub fn escape(&mut self) -> Mode {
        Mode::Exiting
    }
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
        Paragraph::new("Configuration editing is not yet implemented")
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Nodes")
                    .title_alignment(Alignment::Center)
                    .border_set(symbols::border::DOUBLE)
                    .style(THEME.middle),
            )
            .render(area, buf);
    }
}
