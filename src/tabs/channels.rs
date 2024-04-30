use crate::app::Mode;
use crate::theme::THEME;
use crate::{DEVICE_CONFIG, PAGE_SIZE};
use meshtastic::protobufs::Channel;
use ratatui::{prelude::*, widgets::*};
use strum::Display;

#[derive(Debug, Clone, Display, Default)]
enum ChannelDisplayMode {
    #[default]
    List,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelsTab {
    row_index: usize,
    page_size: u16,
    table_contents: Vec<Channel>,
}

impl ChannelsTab {
    pub async fn run(&mut self) {
        self.page_size = *PAGE_SIZE.read().await;

        // get channel list and release lock asap
        {
            let dc = DEVICE_CONFIG.read().await;
            if let Some(config) = dc.clone() {
                self.table_contents = config.channels.values().cloned().collect();
            }
        }
        self.table_contents.sort_by(|a, b| a.index.cmp(&b.index));
    }
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
    pub async fn function_key(&mut self, _num: u8) {
        {}
    }
}

impl Widget for ChannelsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // herein lies the ui code for the tab

        let constraints = vec![
            Constraint::Max(10),
            Constraint::Max(20),
            Constraint::Max(10),
        ];

        let rows: Vec<Row> = self
            .table_contents
            .iter()
            .map(|c| {
                let settings = c.clone().settings.unwrap();
                Row::new(vec![
                    format!("{:02}", c.index),
                    format!("{}", settings.name),
                    format!("{}/{}", settings.uplink_enabled, settings.downlink_enabled),
                ])
            })
            .collect();

        Widget::render(
            Table::new(rows, constraints).block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("About meshtui")
                    .title_alignment(Alignment::Center)
                    .border_set(symbols::border::DOUBLE)
                    .style(THEME.middle),
            ),
            area,
            buf,
        );
    }
}
