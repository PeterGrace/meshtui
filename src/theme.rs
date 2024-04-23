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
}
pub struct NodesTheme {
    pub list: Style,
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
    warning_highlight: Style::new().fg(MENU_COLOR_FOREGROUND).bg(Color::Magenta),
    footer: Style::new()
        .fg(MENU_COLOR_FOREGROUND)
        .bg(MENU_COLOR_BACKGROUND),
    borders: Style::new().fg(BORDER_MID_COLOR_FG).bg(BORDER_MID_COLOR_BG),
    middle: Style::new().fg(MID_COLOR_FG).bg(MID_COLOR_BG),
    nodes: NodesTheme { list: Style::new() },
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
};

//  https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html#
pub const MENU_COLOR_HIGHLIGHT: Color = Color::Red;
pub const MENU_COLOR_FOREGROUND: Color = Color::Black;
pub const MENU_COLOR_BACKGROUND: Color = Color::Gray;

pub const BORDER_MID_COLOR_FG: Color = Color::White;
pub const BORDER_MID_COLOR_BG: Color = Color::Blue;

pub const MID_COLOR_FG: Color = Color::Yellow;
pub const MID_COLOR_BG: Color = Color::Blue;
