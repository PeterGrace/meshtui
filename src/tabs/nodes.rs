use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use itertools::Itertools;
use meshtastic::protobufs::*;
use ratatui::{prelude::*, widgets::*};
use time::OffsetDateTime;
use crate::consts;
use crate::tabs::messages::Message;
use crate::theme::THEME;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodesTab {
    row_index: usize,
    pub node_list: HashMap<u32, NodeInfo>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    vertical_scroll: i32,
}

impl NodesTab {
    pub fn prev_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.node_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    pub fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.node_list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }
}

impl Widget for NodesTab {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // herein lies the ui code for the tab
        let node_list_constraints = vec![
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(40),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
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

                let hops: String = match ni.via_mqtt {
                    true => "MQTT".to_string(),
                    false => ni.hops_away.to_string()
                };
                let since: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u128 - ni.last_heard as u128;
                Row::new(vec![
                    user.id,
                    hops,
                    user.short_name,
                    user.long_name,
                    format!("{}", position.latitude_i as f32 * consts::GPS_PRECISION_FACTOR),
                    format!("{}", position.longitude_i as f32 * consts::GPS_PRECISION_FACTOR),
                    position.altitude.to_string(),
                    format!("{}V", device.voltage),
                    format!("{}%", device.battery_level),
                    format!("{} secs ago", since)
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
            vec!["ID", "Hops", "Short", "Long", "Latitude", "Longitude", "Altitude", "Voltage", "Battery","Last heard"],
        ).set_style(THEME.message_header)
            .bottom_margin(1);


        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .style(THEME.tabs_selected)
            .end_symbol(None);

        StatefulWidget::render(
            Table::new(rows, node_list_constraints)
                .block(block)
                .header(header)
                .highlight_style(THEME.tabs_selected)
            , area, buf, &mut self.table_state);

        StatefulWidget::render(
            scrollbar, area.inner(&Margin { vertical: 1, horizontal: 0 }),
            buf,
            &mut self.scrollbar_state);
    }
}

