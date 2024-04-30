use crate::app::{DeviceConfiguration, Mode};
use crate::theme::THEME;
use crate::DEVICE_CONFIG;
use ratatui::{prelude::*, widgets::*};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Debug, Clone, Default)]
pub struct ConfigTab {
    row_index: usize,
    pub device_config: DeviceConfiguration,
    tab: InnerConfigTabs,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum InnerConfigTabs {
    #[default]
    Device,
    Bluetooth,
    Display,
    LoRa,
    Network,
    Position,
    Power,
}

impl InnerConfigTabs {
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
    fn prev(self) -> Self {
        let current_index = self as usize;
        let prev_index = current_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }
    fn title(self) -> String {
        let tab = self;
        format!(" {tab} ")
    }
}

impl ConfigTab {
    pub async fn run(&mut self) {
        let dc = DEVICE_CONFIG.read().await;
        if let Some(config) = dc.clone() {
            self.device_config = config.clone();
        }
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
    pub fn function_key(&mut self, _num: u8) {
        {}
    }
    pub fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = InnerConfigTabs::iter().map(InnerConfigTabs::title);
        Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .divider("")
            .padding("", "")
            .select(self.tab as usize)
            .render(area, buf);
    }
    pub fn left(&mut self) {
        self.tab = self.tab.prev();
    }
    pub fn right(&mut self) {
        self.tab = self.tab.next();
    }
}

impl Widget for ConfigTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let default_inner_block = Block::default()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .border_set(symbols::border::ROUNDED)
            .style(THEME.middle);

        let display_constraints = vec![Constraint::Min(1), Constraint::Percentage(100)];

        let [bar, field] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(display_constraints)
            .margin(1)
            .areas(area);

        let device_block = default_inner_block.clone().title("Configuration");

        self.render_tabs(bar, buf);
        let pg = match self.tab {
            InnerConfigTabs::Device => format!("{:#?}", self.device_config.device),
            InnerConfigTabs::Bluetooth => format!("{:#?}", self.device_config.bluetooth),
            InnerConfigTabs::Display => format!("{:#?}", self.device_config.display),
            InnerConfigTabs::LoRa => format!("{:#?}", self.device_config.lora),
            InnerConfigTabs::Network => format!("{:#?}", self.device_config.network),
            InnerConfigTabs::Position => format!("{:#?}", self.device_config.position),
            InnerConfigTabs::Power => format!("{:#?}", self.device_config.power),
        };
        Paragraph::new(pg).block(device_block).render(field, buf);
    }
}
