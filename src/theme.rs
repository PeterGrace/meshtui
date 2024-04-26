use ratatui::prelude::*;

pub struct Theme {
    pub root: Style,
    pub tabs: Style,
    pub tabs_selected: Style,
    pub key_binding: KeyBinding,
    pub borders: Style,
    pub middle: Style,
    pub footer: Style,
    pub nodes: NodesTheme,
    pub date_display: Style,
    pub message_header: Style,
    pub message_selected: Style,
    pub warning_highlight: Style,
    pub popup_window: Style,
}
pub struct NodesTheme {
    pub list: Style,
    pub detail: Style
}
pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}
pub const THEME: Theme = Theme {
    root: Style::new().bg(MENU_COLOR_BACKGROUND),
    tabs: Style::new()
        .fg(MENU_COLOR_FOREGROUND)
        .bg(MENU_COLOR_BACKGROUND),
    tabs_selected: Style::new()
        .fg(MENU_COLOR_FOREGROUND)
        .bg(MENU_COLOR_HIGHLIGHT),
    warning_highlight: Style::new().fg(TV_WHITE).bg(TV_GREEN),
    footer: Style::new()
        .fg(MENU_COLOR_FOREGROUND)
        .bg(MENU_COLOR_BACKGROUND),
    borders: Style::new().fg(BORDER_MID_COLOR_FG).bg(BORDER_MID_COLOR_BG),
    middle: Style::new().fg(MID_COLOR_FG).bg(MID_COLOR_BG),
    nodes: NodesTheme {
        list: Style::new(),
        detail: Style::new().bg(MENU_COLOR_HIGHLIGHT),
    },
    key_binding: KeyBinding {
        key: Style::new().fg(Color::Red).bg(MENU_COLOR_BACKGROUND),
        description: Style::new()
            .fg(MENU_COLOR_FOREGROUND)
            .bg(MENU_COLOR_BACKGROUND),
    },
    date_display: Style::new()
        .fg(MENU_COLOR_FOREGROUND)
        .bg(MENU_COLOR_BACKGROUND),
    message_header: Style::new().fg(Color::LightCyan),
    message_selected: Style::new()
        .fg(MENU_COLOR_FOREGROUND)
        .bg(MENU_COLOR_BACKGROUND),
    popup_window: Style::new()
        .fg(TV_WHITE)
        .bg(TV_GREY)

};

//  https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html#
pub const MENU_COLOR_HIGHLIGHT: Color = TV_GREEN;
pub const MENU_COLOR_FOREGROUND: Color = Color::Black;
pub const MENU_COLOR_BACKGROUND: Color = Color::Gray;

pub const BORDER_MID_COLOR_FG: Color = TV_YELLOW;
pub const BORDER_MID_COLOR_BG: Color = TV_BLUE;

pub const MID_COLOR_FG: Color = TV_YELLOW;
pub const MID_COLOR_BG: Color = TV_BLUE;

pub const TV_BLUE: Color = Color::Rgb(0,0,178);
pub const TV_YELLOW: Color = Color::Rgb(200,200,107);
pub const TV_GREEN: Color = Color::Rgb(24,178,24);
pub const TV_WHITE: Color = Color::Rgb(255,255,255);
pub const TV_GREY: Color = Color::Rgb(178,178,178);