use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use time::OffsetDateTime;
use crate::consts;
use crate::packet_handler::MessageEnvelope;
use crate::tabs::nodes::ComprehensiveNode;
use crate::theme::THEME;

#[derive(Debug, Clone, Default)]
pub struct MessagesTab {
    row_index: usize,
    pub messages: Vec<MessageEnvelope>,
    table_state: TableState,
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
            Constraint::Length(20),
            Constraint::Length(32),
            Constraint::Length(32),
            Constraint::Min(50),
        ];

        self.messages.sort_by(|a,b| a.timestamp.cmp(&b.timestamp));
        let rows = self.messages.iter()
            .map(|message| {
                let dt = OffsetDateTime::from_unix_timestamp(message.timestamp as i64).unwrap();
                let mut destination_str = "Unknown".to_string();
                if let Some(destination) = message.clone().destination {
                    destination_str = format!("{}/{}",destination.channel, destination.user.unwrap().short_name);
                }

                Row::new(vec![
                    format!("{}", dt.format(consts::DATE_FORMAT).unwrap()),
                    message.source.user.clone().unwrap().long_name,
                    destination_str,
                    message.clone().message])
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
          vec!["Time", "Source", "Destination", "Message"],
        ).set_style(THEME.message_header)
            .bottom_margin(1);



        StatefulWidget::render(
            Table::new(rows, message_table_constraints)
                .block(block)
                .header(header)
            , area, buf, &mut self.table_state);
    }
}