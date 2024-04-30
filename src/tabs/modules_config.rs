use crate::app::{DeviceConfiguration, Mode};
use crate::theme::THEME;
use ratatui::{prelude::*, widgets::*};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use crate::DEVICE_CONFIG;





#[derive(Debug, Clone, Default)]
pub struct ModulesConfigTab {
    row_index: usize,
    tab: ModuleTabs,
    device_config: DeviceConfiguration
}



#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum ModuleTabs {
    #[default]
    Mqtt,
    Serial,
    ExternalNotification,
    StoreForward,
    RangeTest,
    Telemetry,
    CannedMessage,
    Audio,
    RemoteHardware,
    NeighborInfo,
    AmbientLighting,
    DetectionSensor,
    Paxcounter
}

impl ModuleTabs {
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
        match self {
            tab => format!(" {tab} "),
        }
    }
}

impl ModulesConfigTab {
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
    pub fn function_key(&mut self, num: u8) {
        match num {
            _ => {}
        }
    }
    pub(crate) fn render_sub_modules(&self, area: Rect, buf: &mut Buffer) {
        todo!()
    }
    pub fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = ModuleTabs::iter().map(ModuleTabs::title);
        let inner_tabs = Tabs::new(titles)
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

impl Widget for ModulesConfigTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let default_inner_block = Block::default()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .border_set(symbols::border::ROUNDED)
            .style(THEME.middle);

        let display_constraints = vec![
            Constraint::Min(1),
            Constraint::Percentage(100),
        ];

        let [bar, field] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(display_constraints)
            .margin(1)
            .areas(area);

        let device_block = default_inner_block.clone().title("Configuration");

        self.render_tabs(bar, buf);
        let pg = match self.tab {
            ModuleTabs::Mqtt => { format!("{:#?}", self.device_config.mqtt)}
            ModuleTabs::Serial => {format!("{:#?}", self.device_config.serial)}
            ModuleTabs::ExternalNotification => {format!("{:#?}", self.device_config.external_notification)}
            ModuleTabs::StoreForward => {format!("{:#?}", self.device_config.store_forward)}
            ModuleTabs::RangeTest => {format!("{:#?}", self.device_config.range_test)}
            ModuleTabs::Telemetry => {format!("{:#?}", self.device_config.telemetry)}
            ModuleTabs::CannedMessage => {format!("{:#?}", self.device_config.canned_message)}
            ModuleTabs::Audio => {format!("{:#?}", self.device_config.audio)}
            ModuleTabs::RemoteHardware => {format!("{:#?}", self.device_config.remote_hardware)}
            ModuleTabs::NeighborInfo => {format!("{:#?}", self.device_config.neighbor_info)}
            ModuleTabs::AmbientLighting => {format!("{:#?}", self.device_config.ambient_lighting)}
            ModuleTabs::DetectionSensor => {format!("{:#?}", self.device_config.detection_sensor)}
            ModuleTabs::Paxcounter => {format!("{:#?}", self.device_config.paxcounter)}
        };
        Paragraph::new(pg)
            .block(device_block)
            .render(field, buf);
    }
}

