use crate::app::Mode;
use crate::packet_handler::MessageEnvelope;
use crate::tabs::nodes::ComprehensiveNode;
use crate::theme::THEME;
use crate::{consts, util, PAGE_SIZE};
use circular_buffer::CircularBuffer;
use itertools::Itertools;
use meshtastic::protobufs::{NodeInfo, User};
use ratatui::{prelude::*, widgets::*};
use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct MessagesTab {
    pub messages: CircularBuffer<{ consts::MAX_MSG_RETENTION }, MessageEnvelope>,
    table_state: TableState,
    editing: bool,
    pub page_size: u16,
}

impl MessagesTab {
    pub async fn run(&mut self) {
        self.page_size = *PAGE_SIZE.read().await;

        if self.messages.len() < consts::MAX_MSG_RETENTION {
            self.messages.push_back(MessageEnvelope {
                timestamp: util::get_secs() as u32,
                source: Some(NodeInfo {
                    num: 0,
                    user: Some(User {
                        id: "".to_string(),
                        long_name: "".to_string(),
                        short_name: "".to_string(),
                        macaddr: vec![],
                        hw_model: 0,
                        is_licensed: false,
                        role: 0,
                    }),
                    position: None,
                    snr: 0.0,
                    last_heard: 0,
                    device_metrics: None,
                    channel: 0,
                    via_mqtt: false,
                    hops_away: 0,
                    is_favorite: false,
                }),
                destination: Default::default(),
                channel: Default::default(),
                message: "scrollbarring".to_string(),
                rx_rssi: 0,
                rx_snr: 0.0,
            });
        }
    }
    pub fn escape(&mut self) -> Mode {
        Mode::Exiting
    }
    pub fn enter_key(&mut self) {
        info!("We got the enter key");
        self.editing = !self.editing;
    }
    pub fn prev_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i <= 0 {
                    self.messages.len().saturating_sub(1)
                } else {
                    i.saturating_sub(1)
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
    pub fn prev_page(&mut self) {
        info!("page_size = {}", self.page_size);
        let i = match self.table_state.selected() {
            Some(i) => {
                if i <= self.page_size as usize {
                    0
                } else {
                    i.saturating_sub(self.page_size as usize)
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.messages.len().saturating_sub(1) {
                    0
                } else {
                    i.saturating_add(1)
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
    pub fn next_page(&mut self) {
        info!("page_size = {}", self.page_size);
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.messages.len().saturating_sub(self.page_size as usize) {
                    self.messages.len() - 1
                } else {
                    i.saturating_add(self.page_size as usize)
                }
            }
            None => 0,
        };
        debug!("i is {i}");
        self.table_state.select(Some(i));
    }
}

impl Widget for MessagesTab {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // since this fn is operating on a copy of the messagestab struct, there
        // were only a few ways I could handle perpetuating the page size for PgUp/PgDn.
        let mut page_size;
        {
            page_size = *PAGE_SIZE.try_read().unwrap();
        }
        if page_size != area.height {
            if let Ok(mut ps) = PAGE_SIZE.try_write() {
                *ps = area.height;
            } else {
                info!("write lock failure on page_size");
            }
        }

        let message_table_constraints = vec![
            Constraint::Length(20),
            Constraint::Length(32),
            Constraint::Length(32),
            Constraint::Min(50),
        ];

        let mut message_list = self.messages.to_vec();
        message_list.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        message_list.reverse();
        let rows = message_list
            .iter()
            .map(|message| {
                let dt =
                    OffsetDateTime::from_unix_timestamp(message.clone().timestamp as i64).unwrap();
                let mut destination_str = format!("Ch. {}", &message.channel);

                Row::new(vec![
                    format!("{}", dt.format(consts::DATE_FORMAT).unwrap()),
                    message
                        .clone()
                        .source
                        .unwrap()
                        .user
                        .clone()
                        .unwrap()
                        .long_name,
                    destination_str,
                    message.clone().message,
                ])
            })
            .collect_vec();

        let block = Block::new()
            .borders(Borders::ALL)
            .title("Messages")
            .title_alignment(Alignment::Center)
            .border_set(symbols::border::DOUBLE)
            .style(THEME.middle);

        let header = Row::new(vec!["Time", "Source", "Destination", "Message"])
            .set_style(THEME.message_header)
            .bottom_margin(1);

        StatefulWidget::render(
            Table::new(rows, message_table_constraints)
                .block(block)
                .header(header)
                .highlight_style(THEME.tabs_selected),
            area,
            buf,
            &mut self.table_state,
        );
    }
}
