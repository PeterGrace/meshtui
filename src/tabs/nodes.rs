use crate::app::Preferences;
use crate::consts;
use crate::theme::THEME;
use crate::util::get_secs;
use crate::PREFERENCES;
use geoutils::Location;
use itertools::Itertools;
use meshtastic::protobufs::*;
use pretty_duration::pretty_duration;
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::ops::Div;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct NodesTab {
    row_index: usize,
    pub node_list: HashMap<u32, ComprehensiveNode>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    vertical_scroll: i32,
    pub my_node_id: u32,
    prefs: Preferences,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComprehensiveNode {
    pub node_info: NodeInfo,
    pub last_seen: u64,
    pub neighbors: Vec<Neighbor>,
    pub last_snr: f32,
    pub last_rssi: i32,
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
            Constraint::Max(10),    // ID
            Constraint::Max(5),     // Hops
            Constraint::Max(5),     // ShortName
            Constraint::Max(24),    // LongName
            Constraint::Max(25),    // RF Details
            Constraint::Length(12), // Distance
            Constraint::Length(10), // Latitude
            Constraint::Length(10), // Longitude
            Constraint::Length(10), // Altitude
            Constraint::Length(10), // Voltage
            Constraint::Max(8),     // Battery
            Constraint::Max(20),    // Last Heard
            Constraint::Max(20),    // Last Updated
        ];

        // We sort by last heard, in reverse order, so that the most recent update is at the top.
        let mut node_vec: Vec<ComprehensiveNode> = self.node_list.values().cloned().collect();
        node_vec.sort_by(|a, b| a.last_seen.cmp(&b.last_seen));
        node_vec.reverse();

        let mut my_location: Option<Location> = None;
        if let Some(my_node) = self.node_list.get(&self.my_node_id) {
            if let Some(pos) = my_node.clone().node_info.position {
                let lat = pos.latitude_i as f32 * consts::GPS_PRECISION_FACTOR;
                let lon = pos.longitude_i as f32 * consts::GPS_PRECISION_FACTOR;
                if lat.ne(&0.0) && lon.ne(&0.0) {
                    my_location = Some(Location::new(lat, lon));
                }
            }
        }

        let rows = node_vec
            .iter()
            .map(|cn| {
                let mut add_this_entry: bool = true;
                let user = cn.clone().node_info.user.unwrap_or_else(|| User::default());
                let device = cn
                    .clone()
                    .node_info
                    .device_metrics
                    .unwrap_or_else(|| DeviceMetrics::default());
                let mut position = cn
                    .clone()
                    .node_info
                    .position
                    .unwrap_or_else(|| Position::default());

                let station_lat = position.latitude_i as f32 * consts::GPS_PRECISION_FACTOR;
                let station_lon = position.longitude_i as f32 * consts::GPS_PRECISION_FACTOR;
                let mut distance_str = "".to_string();
                if my_location.is_some() {
                    let station_location = Location::new(station_lat, station_lon);
                    let distance = station_location.distance_to(&my_location.unwrap()).ok();
                    if distance.is_some() {
                        distance_str =
                            format!("{:.3}km", distance.unwrap().meters().div(1000.0_f64));
                    }
                }

                let hops: String = match cn.node_info.via_mqtt {
                    true => "MQTT".to_string(),
                    false => cn.node_info.hops_away.to_string(),
                };

                let mut now_secs = get_secs();
                let mut ni_lastheard_since: u64 = 0;
                let mut ni_lastheard_since_string = "Unknown".to_string();
                let mut update_since_string = "Unknown".to_string();
                ni_lastheard_since = now_secs.saturating_sub(cn.node_info.last_heard as u64);
                if (ni_lastheard_since >= 0) && (ni_lastheard_since != now_secs) {
                    ni_lastheard_since_string =
                        pretty_duration(&Duration::from_secs(ni_lastheard_since), None);
                };
                let mut lastupdate_since: u64 = 0;
                let mut lastupdate_since_string: String = "Unknown".to_string();
                lastupdate_since = now_secs.saturating_sub(cn.last_seen);
                if (lastupdate_since >= 0) && (lastupdate_since != now_secs) {
                    lastupdate_since_string =
                        pretty_duration(&Duration::from_secs(lastupdate_since), None);
                }
                let mut station_lat_str = "".to_string();
                if station_lat.ne(&0.0) {
                    station_lat_str = station_lat.to_string()
                }
                let mut station_lon_str = "".to_string();
                if station_lon.ne(&0.0) {
                    station_lon_str = station_lon.to_string()
                }

                let mut altitude_str = "".to_string();
                if position.altitude.ne(&0) {
                    altitude_str = format!("{}m", position.altitude.to_string());
                };

                let mut voltage_str = "".to_string();
                if device.voltage.gt(&0.0) {
                    let voltage_str = format!("{:.2}V", device.voltage);
                }
                let mut battery_str = "".to_string();
                if device.battery_level.gt(&0) && device.battery_level.le(&100) {
                    battery_str = format!("{:.2}%", device.battery_level);
                };

                let mut rf_str = "".to_string();
                if !cn.node_info.via_mqtt {
                    if cn.last_snr.ne(&0.0) {
                        rf_str = format!("SNR:{:.2}dB / RSSI:{:.0}dB", cn.last_snr, cn.last_rssi);
                    }
                } else {
                    rf_str = format!("MQTT");
                }

                // I don't want to blocking read every loop iteration so we'll cheat and set
                // self.prefs here, avoiding ::new(),::default() adjusting shenanigans.
                if self.prefs.initialized.len() == 0 {
                    let prefs = PREFERENCES.try_read().unwrap();
                    self.prefs = prefs.clone();
                }

                if !self.prefs.show_mqtt && cn.node_info.via_mqtt {
                    add_this_entry = false;
                }
                if user.id.len() == 0 {
                    add_this_entry = false;
                }
                if add_this_entry {
                    Row::new(vec![
                        user.id,
                        hops,
                        user.short_name,
                        user.long_name,
                        rf_str,
                        distance_str,
                        station_lat_str,
                        station_lon_str,
                        altitude_str,
                        voltage_str,
                        battery_str,
                        ni_lastheard_since_string,
                        lastupdate_since_string,
                    ])
                } else {
                    Row::default()
                }
            })
            .collect_vec();

        let header = Row::new(vec![
            "ID",
            "Hops",
            "Short",
            "Long",
            "RF Details",
            "Distance",
            "Latitude",
            "Longitude",
            "Altitude",
            "Voltage",
            "Battery",
            "Last Heard NodeInfo",
            "Last Update",
        ])
        .set_style(THEME.message_header)
        .bottom_margin(1);

        let block = Block::new()
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
                .highlight_style(THEME.tabs_selected),
            area,
            buf,
            &mut self.table_state,
        );

        StatefulWidget::render(
            scrollbar,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            buf,
            &mut self.scrollbar_state,
        );
    }
}
