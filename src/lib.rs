//! This crate will give all the functionalities to read eve logs and parse them
//!
//! Currently it will be able to read out damage and logi logs.
//! It needs to be extended with other log types and the functionalities to read logs from the log
//! directory as they're being written by the game

mod parser;
mod watcher;

pub mod models;

pub use parser::{parse_log_header, parse_log_line};
pub use watcher::{get_log_folder, watch_log_file};
