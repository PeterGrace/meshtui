use ratatui::style::Color;
use time::macros::format_description;
use time::format_description::BorrowedFormatItem;

//  https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html#
pub const MENU_COLOR_HIGHLIGHT: Color = Color::Red;
pub const MENU_COLOR_FOREGROUND: Color = Color::Black;
pub const MENU_COLOR_BACKGROUND: Color = Color::Gray;

pub const BORDER_MID_COLOR_FG: Color = Color::White;
pub const BORDER_MID_COLOR_BG: Color = Color::Blue;

pub const DATE_FORMAT_STR: &'static str = "%Y-%m-%d %H:%M:%S";

pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

