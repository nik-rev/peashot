//! The ferrishot app
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::needless_pass_by_value,
        reason = "relaxed lints in tests"
    )
)]

/// See [`CommandHandler`](crate::config::commands::CommandHandler) for more info
mod command {
    pub use super::config::commands::CommandHandler as Handler;
}

mod clipboard;
mod config;
mod geometry;
mod icons;
mod image;
mod lazy_rect;
mod message;
mod ui;

use config::commands::Command;

use config::Theme;
use message::Message;

pub mod last_region;
pub mod logging;

#[cfg(target_os = "linux")]
pub use clipboard::{CLIPBOARD_DAEMON_ID, run_clipboard_daemon};

pub use config::{Cli, Config, DEFAULT_KDL_CONFIG_STR, DEFAULT_LOG_FILE_PATH};
pub use image::action::SAVED_IMAGE;
pub use image::get_image;
pub use ui::App;
