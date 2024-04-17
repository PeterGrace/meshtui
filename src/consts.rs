use time::macros::format_description;
use time::format_description::BorrowedFormatItem;


pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub const TICK_RATE: f64 = 10.0_f64;
pub const FRAME_RATE: f64 = 2.0_f64;