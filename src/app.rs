use crate::tui;
use crate::tui::Event;
use anyhow::Result;
use std::collections::HashMap;
use crate::consts;
use time::{
    OffsetDateTime,
    format_description::well_known::Rfc3339
};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Constraint::*,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Running,
    Exiting,
}
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    #[default]
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

#[derive(Default)]
pub struct App {
    pub should_quit: bool,
    pub tab: MenuTabs
}

impl App {
    pub async fn run(&mut self) -> Result<()> {
        let mut tui = tui::Tui::new()
            .unwrap()
            .tick_rate(4.0) // 4 ticks per second
            .frame_rate(4.0); // 30 frames per second

        tui.enter(); // Starts event handler, enters raw mode, enters alternate screen

        loop {
            tui.draw(|f| {
                // Deref allows calling `tui.terminal.draw`
                self.ui(f);
            })?;

            if let Some(evt) = tui.next().await {
                if let Event::Key(press) = evt {
                    use KeyCode::*;
                    match press.code {
                        Char('q') | Esc => break,
                        Char('h') | Left => self.prev_tab(),
                        Char('l') | Right => self.next_tab(),
                        _ => {}
                    };
                }
            };

            if self.should_quit {
                break;
            }
        }

        tui.exit(); // stops event handler, exits raw mode, exits alternate screen

        Ok(())
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

    fn ui(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1)
            ])
            .split(frame.size());

        // tabs
        let titles = MenuTabs::iter().map(MenuTabs::title);
        let top_menu = Tabs::new(titles)
                .style(Style::default().fg(consts::MENU_COLOR_FOREGROUND).bg(consts::MENU_COLOR_BACKGROUND))
                .highlight_style(Style::default().fg(consts::MENU_COLOR_FOREGROUND).bg(consts::MENU_COLOR_HIGHLIGHT))
                .divider("")
                .padding("","")
                .select(self.tab as usize)
            ;

        let current_tab: String = format!("{:?}", self.tab);
        let middle = Paragraph::new("MiddleGround")
            .block(
                Block::new()
                .borders(Borders::ALL)
                .title(current_tab)
                .title_alignment(Alignment::Center)
                .border_set(symbols::border::DOUBLE)
                .border_style(Style::default().fg(consts::BORDER_MID_COLOR_FG).bg(consts::BORDER_MID_COLOR_BG))
                .style(Style::default().fg(Color::Yellow).bg(Color::Blue))
                );

        let dt: OffsetDateTime = OffsetDateTime::now_utc();

        let bottom_menu = Paragraph::new(dt.format(consts::DATE_FORMAT).unwrap())
            .block(
                Block::new()
                .style(Style::default().fg(consts::MENU_COLOR_FOREGROUND).bg(consts::MENU_COLOR_BACKGROUND))
                );

        frame.render_widget(top_menu,layout[0]);
        frame.render_widget(middle,layout[1]);
        frame.render_widget(bottom_menu,layout[2]);
    }
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum MenuTabs {
    #[default]
    Messages,
    Nodes,
    Config,
    About
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
