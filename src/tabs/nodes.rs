use crate::app::{Mode, Preferences};
use crate::{consts, util};
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
use meshtastic::protobufs::config::device_config::Role;
use crate::consts::GPS_PRECISION_FACTOR;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum DisplayMode {
    #[default]
    List,
    Detail,
}

#[derive(Debug, Clone, Default)]
pub struct NodesTab {
    //row_index: usize,
    pub node_list: HashMap<u32, ComprehensiveNode>,
    table_state: TableState,
    pub table_contents: Vec<ComprehensiveNode>,
    scrollbar_state: ScrollbarState,
    vertical_scroll: i32,
    pub my_node_id: u32,
    prefs: Preferences,
    pub display_mode: DisplayMode,
    pub selected_node: ComprehensiveNode,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComprehensiveNode {
    pub id: u32,
    pub node_info: NodeInfo,
    pub last_seen: u64,
    pub neighbors: Vec<Neighbor>,
    pub last_snr: f32,
    pub last_rssi: i32,
}

impl ComprehensiveNode {
    pub fn with_id(id: u32) -> Self {
        let mut cn = ComprehensiveNode::default();
        cn.id = id;
        cn

    }
}

impl NodesTab {
    pub fn run(&mut self) {
        if self.prefs.initialized.len() == 0 {
            let prefs = PREFERENCES.try_read().unwrap();
            self.prefs = prefs.clone();
        }

        // We sort by last heard, in reverse order, so that the most recent update is at the top.
        self.table_contents = self.node_list.values().cloned().collect();

        if self.prefs.show_mqtt == false {
            self.table_contents = self
                .table_contents
                .iter()
                .filter_map(|cn| {
                    if cn.clone().node_info.via_mqtt == false {
                        Some(cn.to_owned())
                    } else {
                        None
                    }
                })
                .collect();
        }
        self.table_contents
            .sort_by(|a, b| a.last_seen.cmp(&b.last_seen));
        self.table_contents.reverse();
    }
    pub(crate) fn get_details_for_node(&self) -> Table {
        let cn = self.selected_node.clone();
        let constraints = vec![
            Constraint::Max(20),
            Constraint::Min(0),
            Constraint::Min(0),
            Constraint::Min(0),
            Constraint::Min(0),
        ];
        let mut rows: Vec<Row> = vec![];

        if cn.node_info.via_mqtt {
            rows.push(Row::new(vec!["====(VIA MQTT)===="]).style(THEME.warning_highlight));
        }

        rows.push(Row::new(vec![
            "Node id (num)".to_string(),
            cn.id.to_string(),

        ]));

        //region User-struct display fields
        if cn.node_info.user.is_some() {
            let user = cn.node_info.user.unwrap();

            rows.push(Row::new(vec!["Id".to_string(), user.id.clone()]));


            rows.push(Row::new(vec!["Name (Short)".to_string(),
                                    format!("{} ({})", user.long_name, user.short_name)]));

            rows.push(Row::new(vec!["Hardware Model".to_string(),
                                    format!("{:?}", user.hw_model())]));
            rows.push(Row::new(vec!["Licensed Operator".to_string(),
                                    format!("{}", user.is_licensed)]));
            rows.push(Row::new(vec!["Device Role".to_string(),
                                    format!("{:?}", user.role())]));
        } else {
            rows.push(Row::new(vec![
                "Id* (implied)".to_string(),
                format!("*{:x}",cn.id)
            ]));

        }
        //endregion

        rows.push(Row::new(vec!["Last RF SNR/RSSI".to_string(),
                                format!("{:.2}dB/{:.2}db", cn.last_snr, cn.last_rssi)]));

        //region DeviceMetrics-struct display fields
        if let Some(device_metrics) = cn.node_info.device_metrics {
            if device_metrics.air_util_tx > 0.0 {
                rows.push(Row::new(vec!["Air/TX Utilization".to_string(),
                                        format!("{:.2}%", device_metrics.air_util_tx)]));
            }
            if device_metrics.channel_utilization > 0.0 {
                rows.push(Row::new(vec!["Channel Utilization".to_string(),
                                        format!("{:.2}%", device_metrics.channel_utilization)]));
            }

            if device_metrics.voltage > 0.0 {
                rows.push(Row::new(vec!["Device Voltage".to_string(),
                                        format!("{:.2}V", device_metrics.voltage)]));
            }
            match device_metrics.battery_level {
                1..=100 => {
                    rows.push(Row::new(vec!["Battery Level".to_string(),
                                            format!("{:.2}%", device_metrics.battery_level)]));
                }
                101 => {
                    rows.push(Row::new(vec!["Battery Level".to_string(),
                                            format!("Plugged-in")]));
                }
                _ => {}
            }
        }
        //endregion

        //region Position-struct display fields
        if let Some(position) = cn.node_info.position {
            if position.latitude_i != 0 {
                rows.push(Row::new(vec!["Latitude".to_string(),
                                        format!("{:.2}", position.latitude_i as f32 * (GPS_PRECISION_FACTOR))]));
            }
            if position.longitude_i != 0 {
                rows.push(Row::new(vec!["Longitude".to_string(),
                                        format!("{:.2}", position.longitude_i as f32 * (GPS_PRECISION_FACTOR))]));
            }
            if position.altitude > 0 {
                rows.push(Row::new(vec!["Altitude".to_string(),
                                        format!("{}m", position.altitude)]));
            }
        }
        //endregion

        //region NeighborApp display fields
        if cn.neighbors.len() > 0 {
            rows.push(Row::new(vec!["Neighbors:", "id", "SNR", "Last Seen"]));
            for (i, item) in cn.neighbors.iter().enumerate() {
                let id = self.node_list.get(&item.node_id).unwrap().clone().node_info.user.unwrap().id;
                let snr = format!("{:.2}dB", item.snr);
                let mut last_seen: String = "Unknown".to_string();
                if item.last_rx_time > 0 {
                    last_seen = pretty_duration(&Duration::from_secs(util::get_secs().saturating_sub(item.last_rx_time as u64)), None);
                }
                rows.push(Row::new(vec!["".to_string(), id, snr, last_seen]));
            }
        }
        //endregion

        Table::new(rows, constraints)
            .highlight_style(THEME.tabs_selected)
    }


    pub fn escape(&mut self) -> Mode {
        match self.display_mode {
            DisplayMode::List => Mode::Exiting,
            DisplayMode::Detail => {
                self.display_mode = DisplayMode::List;
                Mode::Running
            }
        }
    }
    pub fn enter_key(&mut self) {
        match self.display_mode {
            DisplayMode::List => {
                if let Some(index) = self.table_state.selected() {
                    self.selected_node = self.table_contents[index].clone();
                    self.display_mode = DisplayMode::Detail
                }
            }
            DisplayMode::Detail => self.display_mode = DisplayMode::List,
        }
    }
    pub fn prev_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i <= 0 {
                    self.table_contents.len() - 1
                } else {
                    i.saturating_sub(1)
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
                if i >= self.table_contents.len() - 1 {
                    0
                } else {
                    i.saturating_add(1)
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

        if self.display_mode == DisplayMode::Detail {
            let popup_block = Block::default()
                .title("Details")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .border_set(symbols::border::DOUBLE)
                .style(THEME.middle);
            let popup_area = crate::app::centered_rect(area, 85, 65);
            let popup_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50)])
                .split(popup_area);

            Widget::render(Clear::default(), area, buf);
            Widget::render(popup_block, popup_area, buf);
            Widget::render(self.get_details_for_node(),
                           crate::app::centered_rect(popup_area,
                                                     95,
                                                     95),
                           buf,
            );
        } else {
            let node_list_constraints = vec![
                Constraint::Max(10),    // ID
                Constraint::Max(5),     // ShortName
                Constraint::Max(25),    // LongName
                Constraint::Max(25),    // RF Details
                Constraint::Max(5),     // Hops
                Constraint::Max(10),     // Neighbors
                Constraint::Length(12), // Distance
                Constraint::Length(10), // Latitude
                Constraint::Length(10), // Longitude
                Constraint::Length(10), // Altitude
                Constraint::Length(10), // Voltage
                Constraint::Max(8),     // Battery
                Constraint::Max(20),    // Last Heard
                Constraint::Max(20),    // Last Updated
            ];

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

            let rows = self
                .table_contents
                .iter()
                .map(|cn| {
                    let mut add_this_entry: bool = true;
                    let mut user_id_str = "Unknown".to_string();
                    let user = cn.clone().node_info.user.unwrap_or_else(|| User::default());
                    if user.id.len() > 0 {
                        user_id_str = user.id;
                    } else {
                        user_id_str = format!("*{:x}", cn.clone().id);
                    }
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
                            rf_str =
                                format!("SNR:{:.2}dB / RSSI:{:.0}dB", cn.last_snr, cn.last_rssi);
                        }
                    } else {
                        rf_str = format!("MQTT");
                    }
                    let neigh_str = format!("{}",cn.neighbors.len());

                    // I don't want to blocking read every loop iteration so we'll cheat and set
                    // self.prefs here, avoiding ::new(),::default() adjusting shenanigans.

                    Row::new(vec![
                        user_id_str,
                        user.short_name,
                        user.long_name,
                        rf_str,
                        hops,
                        neigh_str,
                        distance_str,
                        station_lat_str,
                        station_lon_str,
                        altitude_str,
                        voltage_str,
                        battery_str,
                        ni_lastheard_since_string,
                        lastupdate_since_string,
                    ])
                })
                .collect_vec();

            let header = Row::new(vec![
                "ID",
                "Short",
                "Long",
                "RF Details",
                "Hops",
                "Neighbors",
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
}
