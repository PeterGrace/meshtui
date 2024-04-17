use crate::ipc::IPCMessage;
use crate::tui;
use crate::tui::Event;
use anyhow::Result;
use itertools::Itertools;
use crate::consts;
use color_eyre::eyre::WrapErr;
use crate::theme;
use time::{
    OffsetDateTime,
    format_description::well_known::Rfc3339
};

use crossterm::event::KeyCode;
use crossterm::event::KeyCode::{Down, Esc, Left, Right, Up};
use meshtastic::Message;
use meshtastic::protobufs::*;
use meshtastic::protobufs::telemetry::Variant;
use ratatui::{
    layout::Constraint::*,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};
use ratatui::widgets::{Row, Table};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tui_logger::TuiLoggerWidget;
use crate::tabs::*;
use crate::theme::THEME;
use crate::tui::Event::Render;
use tokio::task;
use tokio::sync::{
    broadcast,
    mpsc,
    RwLock
};
use tokio::task::JoinSet;
use crate::meshtastic_interaction::meshtastic_loop;
#[derive(Debug, Default, Clone, PartialEq)]
pub struct App {
    pub mode: Mode,
    pub tab: MenuTabs,
    pub nodes_tab: NodesTab,
    pub config_tab: ConfigTab,
    pub messages_tab: MessagesTab,
    pub input_mode: InputMode,
    pub cursor_position: usize,
    pub input: String,
    pub event_log: Vec<EventLogItem>
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EventLogItem {
    pub timestamp: String,
    pub log_message: String
}

impl App {
    pub async fn run(&mut self) -> Result<()> {
        let mut tui = tui::Tui::new()
            .unwrap()
            .tick_rate(consts::TICK_RATE)
            .frame_rate(consts::FRAME_RATE);

        tui.enter(); // Starts event handler, enters raw mode, enters alternate screen

        let mut join_set = JoinSet::new();

        let (fromradio_thread_tx, mut fromradio_thread_rx) = mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
        let (toradio_thread_tx, toradio_thread_rx) = mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);

        let to_tx = fromradio_thread_tx.clone();
        join_set.spawn(async move {meshtastic_loop(to_tx).await});

        while self.is_running() {
            self.draw(&mut tui.terminal);
            if let Some(evt) = tui.next().await {
                if let Event::Key(press) = evt {
                    use KeyCode::*;
                    match self.input_mode {
                        InputMode::Normal => {
                            match press.code {
                                Char('q') | Esc => { self.mode = Mode::Exiting; },
                                Char('h') | Left => self.prev_tab(),
                                Char('l') | Right => self.next_tab(),
                                Char('k') | Up => self.prev(),
                                Char('j') | Down => self.next(),
                                _ => {}
                            }
                        },
                        InputMode::Editing => {
                            match press.code {
                                KeyCode::Enter => {},
                                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                                KeyCode::Backspace => {
                                    self.delete_char();
                                }
                                KeyCode::Left => {
                                    self.move_cursor_left();
                                }
                                KeyCode::Right => {
                                    self.move_cursor_right();
                                }
                                KeyCode::Esc => {
                                    self.input_mode = InputMode::Normal;
                                }
                                _ => {}
                            }

                        },


                    }
                }
            };

            if let Ok(packet) = fromradio_thread_rx.try_recv() {
                if let IPCMessage::FromRadio(fr) = packet {
                    if let Some(some_fr) = fr.payload_variant {
                        match some_fr {
                            from_radio::PayloadVariant::Packet(pa) => {
                                if let Some(payload) = pa.payload_variant {
                                    match payload {
                                        mesh_packet::PayloadVariant::Decoded(de) => {
                                            match de.portnum() {
                                                PortNum::TextMessageApp => {}
                                                PortNum::PositionApp => {
                                                    let data = Position::decode(de.payload.as_slice()).unwrap();
                                                    let mut ni = match self.nodes_tab.node_list.contains_key(&de.source) {
                                                        true => self.nodes_tab.node_list.get(&de.source).unwrap().to_owned(),
                                                        false => NodeInfo::default()
                                                    };
                                                    ni.position = Some(data);
                                                    self.nodes_tab.node_list.insert(de.source, ni);
                                                }
                                                PortNum::NodeinfoApp => {}
                                                PortNum::RoutingApp => {}
                                                PortNum::AdminApp => {}
                                                PortNum::WaypointApp => {}
                                                PortNum::ReplyApp => {}
                                                PortNum::PaxcounterApp => {}
                                                PortNum::StoreForwardApp => {}
                                                PortNum::RangeTestApp => {}
                                                PortNum::TelemetryApp => {
                                                    let data = meshtastic::protobufs::Telemetry::decode(de.payload.as_slice()).unwrap();
                                                    if let Some(v) = data.variant {
                                                        match v {
                                                            Variant::DeviceMetrics(dm) => {
                                                                let mut ni = match self.nodes_tab.node_list.contains_key(&de.source) {
                                                                    true => self.nodes_tab.node_list.get(&de.source).unwrap().to_owned(),
                                                                    false => NodeInfo::default()
                                                                };
                                                                ni.device_metrics = Some(dm);
                                                                self.nodes_tab.node_list.insert(de.source, ni);
                                                            }
                                                            Variant::EnvironmentMetrics(_) => {}
                                                            Variant::AirQualityMetrics(_) => {}
                                                            Variant::PowerMetrics(_) => {}
                                                        }
                                                    }
                                                }
                                                PortNum::TracerouteApp => {}
                                                PortNum::NeighborinfoApp => {}
                                                _ => {}
                                            }
                                        }
                                        mesh_packet::PayloadVariant::Encrypted(_) => {}
                                    }
                                }


                            }
                            from_radio::PayloadVariant::MyInfo(mi) => {
                                info!("My node number is {:#?}", mi.my_node_num);
                            },
                            from_radio::PayloadVariant::NodeInfo(ni) => {
                                self.nodes_tab.node_list.insert(ni.num, ni);
                            }
                            from_radio::PayloadVariant::Config(_) => {}
                            from_radio::PayloadVariant::LogRecord(_) => {}
                            from_radio::PayloadVariant::ConfigCompleteId(_) => {}
                            from_radio::PayloadVariant::Rebooted(_) => {}
                            from_radio::PayloadVariant::ModuleConfig(_) => {}
                            from_radio::PayloadVariant::Channel(_) => {}
                            from_radio::PayloadVariant::QueueStatus(_) => {}
                            from_radio::PayloadVariant::XmodemPacket(_) => {}
                            from_radio::PayloadVariant::Metadata(_) => {}
                            from_radio::PayloadVariant::MqttClientProxyMessage(_) => {}
                        }
                    }
                }
            }
        }
        tui.exit(); // stops event handler, exits raw mode, exits alternate screen
        join_set.abort_all();
        Ok(())
    }
    fn draw(&self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        terminal
            .draw(|frame| {
                frame.render_widget(self, frame.size());
            })
            .wrap_err("terminal.draw").unwrap();
        Ok(())
    }
    fn is_running(&self) -> bool {
        self.mode != Mode::Exiting
    }
    fn handle_event(&mut self, event: Event) -> bool {
        true
    }
    fn update(&mut self, action: bool) -> Option<bool> {
        None
    }
    fn prev_tab(&mut self) {
        self.tab = self.tab.prev();
    }

    fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }
    fn prev(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.prev_row(),
            MenuTabs::Messages => self.messages_tab.prev_row(),
            MenuTabs::Config => self.config_tab.prev_row(),
            _ => {}
        }
    }

    fn next(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.next_row(),
            MenuTabs::Messages => self.messages_tab.next_row(),
            MenuTabs::Config => self.config_tab.next_row(),
            _ => {}
        }
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    fn render_event_log(&self, area: Rect, buf: &mut Buffer) {
        let block =
            Block::new()
                .borders(Borders::ALL)
                .title("Event Log")
                .title_alignment(Alignment::Center)
                .border_set(symbols::border::DOUBLE)
                .style(THEME.middle);

        TuiLoggerWidget::default()
            .block(block)
            .render(area, buf)

    }
    fn render_bottom_bar(area: Rect, buf: &mut Buffer) {
        let keys = [
            ("H/←", "Left"),
            ("L/→", "Right"),
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("Q/Esc", "Quit"),
        ];
        let dt: OffsetDateTime = OffsetDateTime::now_utc();

        let mut spans = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
                let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
                [key, desc]
            })
            .collect_vec();
        spans.push(
            Span::styled(
                format!("| {}", dt.format(consts::DATE_FORMAT).unwrap()),
                THEME.date_display
            )
        );
        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }
    pub fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = MenuTabs::iter().map(MenuTabs::title);
        let top_menu = Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .divider("")
            .padding("", "")
            .select(self.tab as usize).render(area, buf);
    }
    pub fn render_selected_tab(&self, area: Rect, buf: &mut Buffer) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.clone().render(area, buf),
            MenuTabs::Messages => self.messages_tab.clone().render(area, buf),
            MenuTabs::Config => self.config_tab.clone().render(area, buf),
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(12),
                Constraint::Length(1)
            ]);
        let [tabs, middle, event_log, bottom_bar] = layout.areas(area);
        Block::new().style(THEME.root).render(area, buf);
        self.render_tabs(tabs, buf);
        self.render_selected_tab(middle, buf);
        self.render_event_log(event_log, buf);
        App::render_bottom_bar(bottom_bar, buf);

    }
}




impl MenuTabs {
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
            Self::About => String::new(),
            tab => format!(" {tab} "),
        }
    }
}
//region "enums"
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Running,
    Exiting,
}
pub enum CurrentlyEditing {
    Key,
    Value,
}
#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum MenuTabs {
    #[default]
    Messages,
    Nodes,
    Config,
    About
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}
//endregion

