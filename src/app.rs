use crate::consts;
use crate::ipc::IPCMessage;
use crate::meshtastic_interaction::meshtastic_loop;
use crate::packet_handler::{process_packet, MessageEnvelope, PacketResponse};
use crate::tabs::nodes::ComprehensiveNode;
use crate::tabs::*;
use crate::theme::THEME;
use crate::tui::Event;
use crate::{tui, util};
use anyhow::Result;
use color_eyre::eyre::WrapErr;
use crossterm::event::KeyCode;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use itertools::Itertools;
use meshtastic::packet::PacketDestination;
use meshtastic::protobufs::Channel;
use meshtastic::types::MeshChannel;
use ratatui::widgets::{Clear, Paragraph};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};
use std::collections::HashMap;
use std::io;

use meshtastic::protobufs::config::*;
use meshtastic::protobufs::module_config::*;

use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use time::OffsetDateTime;
use tokio::sync::mpsc;

use tokio::task::JoinHandle;
use tui_logger::TuiLoggerWidget;

#[derive(Debug, Default, Clone)]
pub struct App {
    pub mode: Mode,
    pub tab: MenuTabs,
    pub nodes_tab: NodesTab,
    pub channels_tab: ChannelsTab,
    pub device_config_tab: ConfigTab,
    pub modules_config_tab: ModulesConfigTab,
    pub messages_tab: MessagesTab,
    pub about_tab: AboutTab,
    pub input_mode: InputMode,
    pub cursor_position: usize,
    pub input: String,
    pub connection: Connection,
    pub user_prefs: Preferences,
}

impl App {
    pub(crate) fn render_send_message_popup(&self, area: Rect, buf: &mut Buffer) {
        let popup_block = Block::default()
            .title("Enter message")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .border_set(symbols::border::DOUBLE)
            .style(THEME.middle);
        let popup_area = centered_rect(area, 60, 25);
        let _popup_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50)])
            .split(popup_area);

        Widget::render(Clear, area, buf);
        Widget::render(popup_block, popup_area, buf);
        Widget::render(
            Paragraph::new(self.input.clone()).style(THEME.message_selected),
            centered_rect(popup_area, 75, 25),
            buf,
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct Preferences {
    pub(crate) initialized: String,
    pub(crate) show_mqtt: bool,
}

#[derive(Debug, Clone, Default)]
pub enum Connection {
    TCP(String, u16),
    Serial(String),
    #[default]
    None,
}

impl App {
    fn chain_hook(&mut self) {
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            disable_raw_mode().unwrap();
            crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
            original_hook(panic);
        }));
    }

    fn escape(&mut self) {
        self.mode = match self.tab {
            MenuTabs::Nodes => self.nodes_tab.escape(),
            MenuTabs::Messages => self.messages_tab.escape(),
            MenuTabs::Channels => self.channels_tab.escape(),
            MenuTabs::DeviceConfig => self.device_config_tab.escape(),
            MenuTabs::ModulesConfig => self.modules_config_tab.escape(),
            MenuTabs::About => self.about_tab.escape(),
        }
    }
    async fn function_key(&mut self, num: u8) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.function_key(num).await,
            MenuTabs::Messages => self.messages_tab.function_key(num),
            MenuTabs::Channels => self.channels_tab.function_key(num).await,
            MenuTabs::DeviceConfig => self.device_config_tab.function_key(num),
            MenuTabs::ModulesConfig => self.modules_config_tab.function_key(num),
            _ => {}
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.chain_hook();
        let mut tui = tui::Tui::new()
            .unwrap()
            .tick_rate(consts::TICK_RATE)
            .frame_rate(consts::FRAME_RATE);

        let _ = tui.enter(); // Starts event handler, enters raw mode, enters alternate screen

        let (mut fromradio_thread_tx, mut fromradio_thread_rx) =
            mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
        let (mut toradio_thread_tx, mut toradio_thread_rx) =
            mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);

        {
            let mut trm = crate::TO_RADIO_MPSC.write().await;
            *trm = Some(toradio_thread_tx.clone());
        }
        let fromradio_tx = fromradio_thread_tx.clone();
        let conn = self.connection.clone();

        let mut join_handle: JoinHandle<Result<()>> = tokio::task::spawn(async move {
            meshtastic_loop(conn, fromradio_tx, toradio_thread_rx).await
        });

        while self.is_running() {
            // execute runs, if needed
            match self.tab {
                MenuTabs::Nodes => self.nodes_tab.run().await,
                MenuTabs::Messages => self.messages_tab.run().await,
                MenuTabs::Channels => self.channels_tab.run().await,
                MenuTabs::DeviceConfig => self.device_config_tab.run().await,
                MenuTabs::ModulesConfig => self.modules_config_tab.run().await,
                _ => {}
            }

            // draw screen
            let _ = self.draw(&mut tui.terminal);

            // process input
            if let Some(Event::Key(press)) = tui.next().await {
                use KeyCode::*;
                match self.input_mode {
                    InputMode::Normal => match press.code {
                        Char('q') | Esc => self.escape(),
                        Char('h') | Left => self.left(),
                        Char('l') | Right => self.right(),
                        Char('k') | Up => self.prev(),
                        Char('j') | Down => self.next(),
                        PageUp => self.prev_page(),
                        PageDown => self.next_page(),
                        KeyCode::Enter => self.enter_key().await,
                        KeyCode::BackTab => self.prev_tab(),
                        KeyCode::Tab => self.next_tab(),
                        KeyCode::F(n) => self.function_key(n).await,
                        _ => {}
                    },
                    InputMode::Editing => match press.code {
                        KeyCode::Enter => self.enter_key().await,
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
                    },
                }
            };

            // execute action logic
            if let Ok(packet) = fromradio_thread_rx.try_recv() {
                let update = process_packet(packet, self.nodes_tab.node_list.clone()).await;
                if update.is_some() {
                    // we received an update on a node
                    match update.unwrap() {
                        PacketResponse::NodeUpdate(id, cn) => {
                            self.nodes_tab.node_list.insert(id, cn);
                        }
                        PacketResponse::InboundMessage(envelope) => {
                            if let Some(cn) = self
                                .nodes_tab
                                .node_list
                                .get(&envelope.clone().source.unwrap().num)
                            {
                                let mut ncn = cn.clone();
                                ncn.last_rssi = envelope.rx_rssi;
                                ncn.last_snr = envelope.rx_snr;
                                self.nodes_tab
                                    .node_list
                                    .insert(envelope.clone().source.unwrap().num, ncn);
                            }
                            self.messages_tab.messages.push_back(envelope);
                        }
                        PacketResponse::UserUpdate(id, user) => {
                            if let Some(cn) = self.nodes_tab.node_list.get(&id) {
                                let mut ncn = cn.clone();
                                ncn.node_info.user = Some(user);
                                ncn.last_seen = util::get_secs();
                                self.nodes_tab.node_list.insert(id, ncn);
                            } else {
                                let mut cn = ComprehensiveNode::with_id(id);
                                cn.node_info.user = Some(user);
                                cn.last_seen = util::get_secs();
                                self.nodes_tab.node_list.insert(id, cn);
                            }
                        }
                        PacketResponse::OurAddress(id) => {
                            self.nodes_tab.my_node_id = id;
                            self.nodes_tab.my_node_id = id;
                        }
                    }
                }
            }

            // tend to our threads
            if join_handle.is_finished() {
                match join_handle.await {
                    Ok(r) => match r {
                        Ok(_) => {
                            unreachable!()
                        }
                        Err(e) => {
                            error!("Comms thread exited, restarting.  Err: {e}");
                            (fromradio_thread_tx, fromradio_thread_rx) =
                                mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
                            (toradio_thread_tx, toradio_thread_rx) =
                                mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
                            let fromradio_tx = fromradio_thread_tx.clone();
                            {
                                let mut trm = crate::TO_RADIO_MPSC.write().await;
                                *trm = Some(toradio_thread_tx.clone());
                            }
                            let conn = self.connection.clone();
                            join_handle = tokio::task::spawn(async move {
                                meshtastic_loop(conn, fromradio_tx, toradio_thread_rx).await
                            });
                        }
                    },
                    Err(e) => {
                        panic!("JoinError: {e}");
                    }
                }
            }
        }
        let _ = tui.exit(); // stops event handler, exits raw mode, exits alternate screen
        join_handle.abort();
        Ok(())
    }
    fn draw(&self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        terminal
            .draw(|frame| {
                frame.render_widget(self, frame.size());
            })
            .wrap_err("terminal.draw")
            .unwrap();
        Ok(())
    }
    fn is_running(&self) -> bool {
        self.mode != Mode::Exiting
    }
    fn prev_tab(&mut self) {
        self.tab = self.tab.prev();
    }

    fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }
    fn left(&mut self) {
        match self.tab {
            MenuTabs::DeviceConfig => self.device_config_tab.left(),
            MenuTabs::ModulesConfig => self.modules_config_tab.left(),
            _ => {}
        }
    }
    fn right(&mut self) {
        match self.tab {
            MenuTabs::DeviceConfig => self.device_config_tab.right(),
            MenuTabs::ModulesConfig => self.modules_config_tab.right(),
            _ => {}
        }
    }
    fn prev(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.prev_row(),
            MenuTabs::Messages => self.messages_tab.prev_row(),
            MenuTabs::Channels => self.channels_tab.prev_row(),
            MenuTabs::DeviceConfig => self.device_config_tab.prev_row(),
            MenuTabs::ModulesConfig => self.modules_config_tab.prev_row(),
            MenuTabs::About => self.about_tab.prev_row(),
        }
    }
    fn prev_page(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.prev_page(),
            MenuTabs::Messages => self.messages_tab.prev_page(),
            MenuTabs::DeviceConfig => {}
            MenuTabs::ModulesConfig => {}
            MenuTabs::About => {}
            _ => {}
        }
    }

    fn next(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.next_row(),
            MenuTabs::Messages => self.messages_tab.next_row(),
            MenuTabs::Channels => self.channels_tab.next_row(),
            MenuTabs::DeviceConfig => self.device_config_tab.next_row(),
            MenuTabs::ModulesConfig => self.modules_config_tab.next_row(),
            MenuTabs::About => self.about_tab.next_row(),
        }
    }
    fn next_page(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.next_page(),
            MenuTabs::Messages => self.messages_tab.next_page(),
            MenuTabs::DeviceConfig => self.device_config_tab.next_row(),
            MenuTabs::ModulesConfig => self.modules_config_tab.next_row(),
            MenuTabs::About => self.about_tab.next_row(),
            _ => {}
        }
    }

    async fn enter_key_messages(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                self.input_mode = InputMode::Editing;
            }
            InputMode::Editing => {
                if !self.input.is_empty() {
                    info!("Sending message {} to LongFast", self.input.clone());
                    let message = MessageEnvelope {
                        timestamp: 0,
                        source: None,
                        destination: PacketDestination::Broadcast,
                        channel: MeshChannel::new(0).unwrap(),
                        message: self.input.clone(),
                        rx_rssi: 0,
                        rx_snr: 0.0,
                    };
                    if let Err(e) = util::send_to_radio(IPCMessage::SendMessage(message)).await {
                        error!("Unable to send message to node: {e}");
                    }
                }
                self.input = "".to_string();
                self.input_mode = InputMode::Normal;
            }
        }
    }

    async fn enter_key(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.enter_key(),
            MenuTabs::Messages => self.enter_key_messages().await,
            MenuTabs::Channels => self.channels_tab.enter_key(),
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

    fn render_event_log(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title("Event Log")
            .title_alignment(Alignment::Center)
            .border_set(symbols::border::DOUBLE)
            .style(THEME.middle);

        TuiLoggerWidget::default().block(block).render(area, buf)
    }

    fn render_bottom_bar(area: Rect, buf: &mut Buffer) {
        let keys = [
            ("H/←", "Left"),
            ("L/→", "Right"),
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("Enter", "Interact/Send"),
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
        spans.push(Span::styled(
            format!("| {}", dt.format(consts::DATE_FORMAT).unwrap()),
            THEME.date_display,
        ));
        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }

    pub fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = MenuTabs::iter().map(MenuTabs::title);
        Tabs::new(titles)
            .style(THEME.tabs)
            .highlight_style(THEME.tabs_selected)
            .divider("")
            .padding("", "")
            .select(self.tab as usize)
            .render(area, buf);
    }

    pub fn render_selected_tab(&self, area: Rect, buf: &mut Buffer) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.clone().render(area, buf),
            MenuTabs::Messages => self.messages_tab.clone().render(area, buf),
            MenuTabs::Channels => self.channels_tab.clone().render(area, buf),
            MenuTabs::DeviceConfig => self.device_config_tab.clone().render(area, buf),
            MenuTabs::ModulesConfig => self.modules_config_tab.clone().render(area, buf),
            MenuTabs::About => self.about_tab.render(area, buf),
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
                Constraint::Length(1),
            ]);
        let [tabs, middle, event_log, bottom_bar] = layout.areas(area);
        Block::new().style(THEME.root).render(area, buf);
        self.render_tabs(tabs, buf);
        match self.input_mode {
            InputMode::Editing => self.render_send_message_popup(middle, buf),
            InputMode::Normal => self.render_selected_tab(middle, buf),
        }
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
        format!(" {self} ")
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Running,
    Exiting,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum MenuTabs {
    #[default]
    Messages,
    Nodes,
    Channels,
    DeviceConfig,
    ModulesConfig,
    About,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

pub(crate) fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[derive(Clone, Default, Debug)]
pub struct DeviceConfiguration {
    pub device: DeviceConfig,
    pub bluetooth: BluetoothConfig,
    pub display: DisplayConfig,
    pub lora: LoRaConfig,
    pub network: NetworkConfig,
    pub position: PositionConfig,
    pub power: PowerConfig,
    pub mqtt: MqttConfig,
    pub serial: SerialConfig,
    pub external_notification: ExternalNotificationConfig,
    pub store_forward: StoreForwardConfig,
    pub range_test: RangeTestConfig,
    pub telemetry: TelemetryConfig,
    pub canned_message: CannedMessageConfig,
    pub audio: AudioConfig,
    pub remote_hardware: RemoteHardwareConfig,
    pub neighbor_info: NeighborInfoConfig,
    pub ambient_lighting: AmbientLightingConfig,
    pub detection_sensor: DetectionSensorConfig,
    pub paxcounter: PaxcounterConfig,
    pub channels: HashMap<i32, Channel>,
    pub last_update: u64,
}
