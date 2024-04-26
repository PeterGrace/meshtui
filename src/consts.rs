use ratatui::{prelude::*, widgets::*};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub const TICK_RATE: f64 = 10.0_f64;
pub const FRAME_RATE: f64 = 2.0_f64;

pub const MPSC_BUFFER_SIZE: usize = 100_usize;
pub const GPS_PRECISION_FACTOR: f32 = 0.0000001_f32;
pub const MAX_MSG_RETENTION: usize = 128_usize;


