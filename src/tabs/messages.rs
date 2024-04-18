use crate::consts::DATE_FORMAT;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use time::OffsetDateTime;
use crate::theme::THEME;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessagesTab {
    row_index: usize,
    pub messages: Vec<Message>,
    table_state: TableState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub time: OffsetDateTime,
    pub source: String,
    pub message: String,
}

impl MessagesTab {
    pub fn enter_key(&mut self) {}
    pub fn prev_row(&mut self) {
        self.row_index = self.row_index.saturating_sub(1);
    }

    pub fn next_row(&mut self) {
        self.row_index = self.row_index.saturating_add(1);
    }
}

impl Widget for MessagesTab {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // herein lies the ui code for the tab

        let message_table_constraints = vec![
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(0),
        ];

        let mut TEST_MESSAGES: Vec<Message> = vec![Message {
            time: OffsetDateTime::now_utc(),
            source: "Foobar".to_string(),
            message: "Bazbat".to_string()
        }];
        TEST_MESSAGES.push(Message {
            time: OffsetDateTime::now_utc(),
            source: "RandomId".to_string(),
            message: "RandomMessage".to_string()
        });

        let rows = TEST_MESSAGES.iter()
            .map(|message| {
                Row::new(vec![
                    "2024-99-99 99:99:99",
                    message.source.as_str(),
                    message.message.as_str()])
            })
            .collect_vec();

        let block =
            Block::new()
                .borders(Borders::ALL)
                .title("Messages")
                .title_alignment(Alignment::Center)
                .border_set(symbols::border::DOUBLE)
                .style(THEME.middle);

        let header = Row::new(
          vec!["Time", "Source", "Message"],
        ).set_style(THEME.message_header)
            .bottom_margin(1);



        StatefulWidget::render(
            Table::new(rows, message_table_constraints)
                .block(block)
                .header(header)
            , area, buf, &mut self.table_state);
    }
}