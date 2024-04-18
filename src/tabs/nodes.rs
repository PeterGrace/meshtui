use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use itertools::Itertools;
use meshtastic::protobufs::*;
use ratatui::{prelude::*, widgets::*};
use time::{OffsetDateTime};
use crate::util::get_secs;
use crate::consts;
use crate::tabs::messages::Message;
use crate::theme::THEME;
use pretty_duration::pretty_duration;
use std::time::Duration;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodesTab {
    row_index: usize,
    pub node_list: HashMap<u32, ComprehensiveNode>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    vertical_scroll: i32,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComprehensiveNode {
    pub node_info: NodeInfo,
    pub last_seen: u64,
    pub neighbors: Vec<Neighbor>
}

impl NodesTab {
    pub fn enter_key(&mut self) {}
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
            Constraint::Min(10),
        ];

        // We sort by last heard, in reverse order, so that the most recent update is at the top.
        let mut node_vec: Vec<ComprehensiveNode> = self.node_list.values().cloned().collect();
        node_vec.sort_by(|a,b| a.last_seen.cmp(&b.last_seen));
        node_vec.reverse();

        let rows = node_vec.iter()
            .map(|cn| {
                let user = cn.clone().node_info.user.unwrap_or_else(|| User::default());
                let device = cn.clone().node_info.device_metrics.unwrap_or_else(|| DeviceMetrics::default());
                let mut position = cn.clone().node_info.position.unwrap_or_else(|| Position::default());
                if position.precision_bits <= 0 {
                    position.precision_bits = 1;
                }

                let hops: String = match cn.node_info.via_mqtt {
                    true => "MQTT".to_string(),
                    false => cn.node_info.hops_away.to_string()
                };

                let mut now_secs = get_secs();
                let mut ni_lastheard_since: u64 = 0;
                let mut ni_lastheard_since_string = "Unknown".to_string();
                let mut update_since_string = "Unknown".to_string();
                ni_lastheard_since = now_secs.saturating_sub(cn.node_info.last_heard as u64);
                if (ni_lastheard_since >= 0) && (ni_lastheard_since != now_secs) {
                    ni_lastheard_since_string = pretty_duration(&Duration::from_secs(ni_lastheard_since), None);
                };
                let mut lastupdate_since: u64 = 0;
                let mut lastupdate_since_string: String = "Unknown".to_string();
                lastupdate_since = now_secs.saturating_sub(cn.last_seen);
                if (lastupdate_since >= 0) && (lastupdate_since != now_secs) {
                    lastupdate_since_string = pretty_duration(&Duration::from_secs(lastupdate_since), None);
                }


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
                    lastupdate_since_string,
                    ni_lastheard_since_string
                ])
            })
            .collect_vec();

        let header = Row::new(
            vec!["ID", "Hops", "Short", "Long", "Latitude", "Longitude", "Altitude", "Voltage", "Battery","Last Updated", "Last Heard NodeInfo"],
        ).set_style(THEME.message_header)
            .bottom_margin(1);


        let block =
            Block::new()
                .borders(Borders::ALL)
                .title("Nodes")
                .title_alignment(Alignment::Center)
                .border_set(symbols::border::DOUBLE)
                .style(THEME.middle);


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

