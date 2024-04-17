use std::collections::HashMap;
use itertools::Itertools;
use meshtastic::protobufs::*;
use ratatui::{prelude::*, widgets::*};
use time::OffsetDateTime;
use crate::tabs::messages::Message;
use crate::theme::THEME;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodesTab {
    row_index: usize,
    pub node_list: HashMap<u32, NodeInfo>,
    table_state: TableState,
}

impl NodesTab {
    pub fn prev_row(&mut self) {
        self.row_index = self.row_index.saturating_sub(1);
    }

    pub fn next_row(&mut self) {
        self.row_index = self.row_index.saturating_add(1);
    }
}
impl Widget for NodesTab {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // herein lies the ui code for the tab
        let node_list_constraints = vec![
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Min(50),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(10),
        ];

        let rows = self.node_list.iter()
            .map(|(id, ni)| {
                let user = ni.clone().user.unwrap_or_else(|| User::default());
                let device = ni.clone().device_metrics.unwrap_or_else(|| DeviceMetrics::default());
                let mut position = ni.clone().position.unwrap_or_else(|| Position::default());
                if position.precision_bits <= 0 {
                    position.precision_bits = 1;
                }
                Row::new(vec![
                    user.id,
                    user.short_name,
                    user.long_name,
                    format!("{}",position.latitude_i as f32 *0.0000001),
                    format!("{}",position.longitude_i as f32 * 0.0000001),
                    position.altitude.to_string(),
                    format!("{}V", device.voltage),
                    format!("{}%",device.battery_level)
            ])
            })
            .collect_vec();

        let block =
            Block::new()
                .borders(Borders::ALL)
                .title("Nodes")
                .title_alignment(Alignment::Center)
                .border_set(symbols::border::DOUBLE)
                .style(THEME.middle);

        let header = Row::new(
            vec!["ID", "Short", "Long", "Latitude", "Longitude", "Altitude", "Voltage", "Battery" ],
        ).set_style(THEME.message_header)
            .bottom_margin(1);



        StatefulWidget::render(
            Table::new(rows, node_list_constraints)
                .block(block)
                .header(header)
            , area, buf, &mut self.table_state);
    }
}