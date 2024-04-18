use crate::ipc::IPCMessage;
use crate::{tui, util};
use crate::tui::Event;
use anyhow::Result;
use crate::consts;
use color_eyre::eyre::WrapErr;
use time::OffsetDateTime;

use crossterm::event::{KeyCode, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tui_logger::TuiLoggerWidget;
use crate::tabs::*;
use crate::theme::THEME;
use tokio::sync::{
    mpsc,
};
use tokio::task::{JoinHandle};
use crate::meshtastic_interaction::meshtastic_loop;
use std::io;
use crate::packet_handler::{process_packet, PacketResponse};
use crate::tabs::nodes::ComprehensiveNode;

#[derive(Debug, Default, Clone)]
pub struct App {
    pub mode: Mode,
    pub tab: MenuTabs,
    pub nodes_tab: NodesTab,
    pub config_tab: ConfigTab,
    pub messages_tab: MessagesTab,
    pub input_mode: InputMode,
    pub cursor_position: usize,
    pub input: String,
    pub connection: Connection,
    pub user_prefs: Preferences
}

#[derive(Debug, Clone, Default)]
pub struct Preferences {
    pub(crate) initialized: String,
    pub(crate) show_mqtt: bool
}
#[derive(Debug, Clone, Default)]
pub enum Connection {
    TCP(String, u16),
    Serial(String),
    #[default]
    None
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

    pub async fn run(&mut self) -> Result<()> {
        self.chain_hook();
        let mut tui = tui::Tui::new()
            .unwrap()
            .tick_rate(consts::TICK_RATE)
            .frame_rate(consts::FRAME_RATE);

        tui.enter(); // Starts event handler, enters raw mode, enters alternate screen



        let (mut fromradio_thread_tx, mut fromradio_thread_rx) = mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
        let (mut toradio_thread_tx, mut toradio_thread_rx) = mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);

        let fromradio_tx = fromradio_thread_tx.clone();
        let conn = self.connection.clone();

        let mut join_handle: JoinHandle<Result<()>> = tokio::task::spawn(async move { meshtastic_loop(conn, fromradio_tx, toradio_thread_rx).await });

        while self.is_running() {
            // draw screen
            self.draw(&mut tui.terminal);

            // process input
            if let Some(evt) = tui.next().await {
                if let Event::Key(press) = evt {
                    use KeyCode::*;
                    match self.input_mode {
                        InputMode::Normal => {
                            match press.code {
                                Char('q') | Esc => { self.mode = Mode::Exiting; }
                                Char('h') | Left => self.prev_tab(),
                                Char('l') | Right => self.next_tab(),
                                Char('k') | Up => self.prev(),
                                Char('j') | Down => self.next(),
                                KeyCode::Enter => self.enter_key(),
                                _ => {}
                            }
                        }
                        InputMode::Editing => {
                            match press.code {
                                KeyCode::Enter => {}
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
                        }
                    }
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
                            if let Some(cn) = self.nodes_tab.node_list.get(&envelope.source.num) {
                                let mut ncn = cn.clone();
                                ncn.last_rssi = envelope.rx_rssi;
                                ncn.last_snr = envelope.rx_snr;
                                self.nodes_tab.node_list.insert(envelope.source.num, ncn);
                            }
                            self.messages_tab.messages.push(envelope);
                        }
                        PacketResponse::UserUpdate(id, user) => {
                            if let Some(cn) = self.nodes_tab.node_list.get(&id) {
                                let mut ncn = cn.clone();
                                ncn.node_info.user = Some(user);
                                ncn.last_seen = util::get_secs();
                                self.nodes_tab.node_list.insert(id, ncn);
                            } else {
                                let mut cn = ComprehensiveNode::default();
                                cn.node_info.user = Some(user);
                                cn.last_seen = util::get_secs();
                                self.nodes_tab.node_list.insert(id, cn);
                            }


                        }
                        PacketResponse::OurAddress(id) => {
                            self.nodes_tab.my_node_id = id;
                        }
                    }
                }
            }

            // tend to our threads
            if join_handle.is_finished() {
                if let Err(e) = join_handle.await {
                    error!("Comms thread exited, restarting.  Err: {e}");
                    (fromradio_thread_tx, fromradio_thread_rx) = mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
                    (toradio_thread_tx, toradio_thread_rx) = mpsc::channel::<IPCMessage>(consts::MPSC_BUFFER_SIZE);
                    let fromradio_tx = fromradio_thread_tx.clone();
                    let conn = self.connection.clone();
                    join_handle = tokio::task::spawn(async move { meshtastic_loop(conn, fromradio_tx, toradio_thread_rx).await });
                }


            }
        }
        tui.exit(); // stops event handler, exits raw mode, exits alternate screen
        join_handle.abort();
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

    fn enter_key(&mut self) {
        match self.tab {
            MenuTabs::Nodes => self.nodes_tab.enter_key(),
            MenuTabs::Messages => self.messages_tab.enter_key(),
            MenuTabs::Config => self.config_tab.enter_key(),
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
                THEME.date_display,
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
                Constraint::Length(1),
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
    Config,
    About,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

