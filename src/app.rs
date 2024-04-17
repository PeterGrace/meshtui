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
use crossterm::event::KeyCode::{Esc, Left, Right};
use ratatui::{
    layout::Constraint::*,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use crate::tabs::*;
use crate::theme::THEME;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct App {
    pub mode: Mode,
    pub tab: MenuTabs,
    pub nodes_tab: NodesTab,
    pub config_tab: ConfigTab,
    pub messages_tab: MessagesTab
}

impl App {
    pub async fn run(&mut self) -> Result<()> {
        let mut tui = tui::Tui::new()
            .unwrap()
            .tick_rate(consts::TICK_RATE)
            .frame_rate(consts::FRAME_RATE);

        tui.enter(); // Starts event handler, enters raw mode, enters alternate screen

        while self.is_running() {
            self.draw(&mut tui.terminal);

            if let Some(evt) = tui.next().await {
                if let Event::Key(press) = evt {
                    use KeyCode::*;
                    match press.code {
                        Char('q') | Esc => { self.mode = Mode::Exiting; },
                        Char('h') | Left => self.prev_tab(),
                        Char('l') | Right => self.next_tab(),
                        _ => {}
                    };
                }
            };
        }
        tui.exit(); // stops event handler, exits raw mode, exits alternate screen

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
            MenuTabs::Nodes => self.nodes_tab.render(area, buf),
            MenuTabs::Messages => self.messages_tab.render(area, buf),
            MenuTabs::Config => self.config_tab.render(area, buf),
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
                Constraint::Length(1)
            ]);
        let [tabs, middle, bottom_bar] = layout.areas(area);
        Block::new().style(THEME.root).render(area, buf);
        self.render_tabs(tabs, buf);
        self.render_selected_tab(middle, buf);
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
//endregion