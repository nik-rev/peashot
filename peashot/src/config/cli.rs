//! Parse the command line arguments passed to ferrishot
use std::time::Duration;
use std::{path::PathBuf, sync::LazyLock};

use clap::{Parser, ValueHint};
use etcetera::BaseStrategy as _;

use crate::lazy_rect::LazyRectangle;

use anstyle::{AnsiColor, Effects};

/// Styles for the CLI
const STYLES: clap::builder::Styles = clap::builder::Styles::styled()
    .header(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::BrightGreen.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::BrightCyan.on_default())
    .error(AnsiColor::BrightRed.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::BrightCyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::BrightYellow.on_default().effects(Effects::BOLD));

/// Ferrishot is a powerful screenshot app written in Rust
#[derive(Parser, Debug)]
#[command(version, styles = STYLES, long_about = None)]
#[expect(clippy::struct_excessive_bools, reason = "normal for CLIs")]
pub struct Cli {
    /// Instead of taking a screenshot of the desktop, open this image instead
    //
    // NOTE: Currently disabled because if the screenshot is not the same size as the desktop,
    // it will cause bugs as we consider 0,0 in the Canvas to be the origin but it is not necessarily,
    // when the desktop and the image are not the same size
    //
    // TODO: Fix this argument
    //
    #[arg(hide = true, value_hint = ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    //
    // --- Options ---
    //
    /// Open with a region pre-selected
    ///
    /// Format: `<width>x<height>+<top-left-x>+<top-left-y>`
    ///
    /// Each value can be absolute.
    /// - 550 for `x` means top-left corner starts after 550px
    /// - 100 for `height` means it will be 100px tall
    ///
    /// Each can also be relative to the height (for `y` and `height`) or width (for `width` and `x`)
    /// - 0.2 for `width` means it region takes up 20% of the width of the image.
    /// - 0.5 for `y` means the top-left corner will be at the vertical center
    ///
    /// The format can also end with 1 or 2 percentages, which shifts the region relative to the region's size
    /// - If `width` is `250`, end region with `+30%` to move right by 75px or `-40%` to move left by 100px
    /// - Supplying 2 percentage at the end like `+30%-10%`, the 1st affects x-offset and the 2nd affects y-offset
    ///
    /// With the above syntax, you can create all the regions you want.
    /// - `100x1.0+0.5+0-50%`: Create a 100px wide, full height, horizontally centered region
    /// - `1.0x1.0+0+0`: Create a region that spans the full screen. You can use alias `full` for this
    #[arg(
        short,
        long,
        value_name = "WxH+X+Y",
        value_hint = ValueHint::Other
    )]
    pub region: Option<LazyRectangle>,

    /// Use last region
    #[arg(short, long, conflicts_with = "region")]
    pub last_region: bool,

    /// Accept capture and perform the action as soon as a selection is made
    ///
    /// If holding `ctrl` while you are releasing the left mouse button on the first selection,
    /// the behavior is cancelled
    ///
    /// It's quite useful to run ferrishot, select a region and have it instantly be copied to the
    /// clipboard for example.
    ///
    /// In 90% of situations you won't want to do much post-processing of
    /// the region and this makes that experience twice as fast. You can always opt-out with `ctrl`
    ///
    /// Using this option with `--region` or `--last-region` will run ferrishot in 'headless mode',
    /// without making a new window.
    #[arg(short, long, value_name = "ACTION")]
    pub accept_on_select: Option<crate::image::action::Command>,

    /// Wait this long before launch
    #[arg(
        short,
        long,
        value_name = "MILLISECONDS",
        value_parser = |s: &str| s.parse().map(Duration::from_millis),
        value_hint = ValueHint::Other
    )]
    pub delay: Option<Duration>,

    /// Save image to path
    #[arg(
        short,
        long,
        value_name = "PATH",
        long_help = "Instead of opening a file picker to save the screenshot, save it to this path instead",
        value_hint = ValueHint::FilePath
    )]
    pub save_path: Option<PathBuf>,

    //
    // --- Config ---
    //
    /// Dump default config
    #[arg(
        help_heading = "Config",
        short = 'D',
        long,
        help = format!("Write the default config to {}",  DEFAULT_CONFIG_FILE_PATH.display()),
        long_help = format!("Write contents of the default config to {}", DEFAULT_CONFIG_FILE_PATH.display()),
    )]
    pub dump_default_config: bool,

    /// Use the provided config file
    #[arg(
        help_heading = "Config",
        short = 'C',
        long,
        value_name = "FILE.KDL",
        default_value_t = DEFAULT_CONFIG_FILE_PATH.to_string_lossy().to_string(),
        value_hint = ValueHint::FilePath
    )]
    pub config_file: String,

    //
    // --- Output
    //
    /// Run in silent mode
    #[arg(
        help_heading = "Output",
        short = 'S',
        long,
        long_help = "Run in silent mode. Do not print anything"
    )]
    pub silent: bool,

    /// Print in JSON format
    #[arg(help_heading = "Output", short, long, conflicts_with = "silent")]
    pub json: bool,

    //
    // --- Debug ---
    //
    // Requires ferrishot to be compiled with `debug` for them to show up in the CLI help
    //
    /// Choose a miniumum level at which to log
    #[arg(
        help_heading = "Debug",
        long,
        value_name = "LEVEL",
        default_value = "error",
        long_help = "Choose a miniumum level at which to log. [error, warn, info, debug, trace, off]",
        hide = !cfg!(feature = "debug")
    )]
    pub log_level: log::LevelFilter,

    /// Log to standard error instead of file
    #[arg(
        help_heading = "Debug",
        long,
        conflicts_with = "silent",
        hide = !cfg!(feature = "debug")
    )]
    pub log_stderr: bool,

    /// Path to the log file
    #[arg(
        help_heading = "Debug",
        long,
        value_name = "FILE",
        default_value_t = DEFAULT_LOG_FILE_PATH.to_string_lossy().to_string(),
        value_hint = ValueHint::FilePath,
        hide = !cfg!(feature = "debug")
    )]
    pub log_file: String,

    /// Filter for specific Rust module or crate, instead of showing logs from all crates
    #[arg(
        help_heading = "Debug",
        long,
        value_name = "FILTER",
        value_hint = ValueHint::Other,
        hide = !cfg!(feature = "debug")
    )]
    pub log_filter: Option<String>,

    /// Launch in debug mode (F12)
    #[arg(
        help_heading = "Debug",
        long,
        hide = !cfg!(feature = "debug")
    )]
    pub debug: bool,
}

/// Represents the default location of the config file
static DEFAULT_CONFIG_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    etcetera::choose_base_strategy().map_or_else(
        |err| {
            log::warn!("Could not determine the config directory: {err}");
            PathBuf::from("ferrishot.kdl")
        },
        |strategy| strategy.config_dir().join("ferrishot.kdl"),
    )
});

/// Represents the default location of the config file
pub static DEFAULT_LOG_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    etcetera::choose_base_strategy().map_or_else(
        |err| {
            log::warn!("Could not determine the config directory: {err}");
            PathBuf::from("ferrishot.log")
        },
        |strategy| strategy.cache_dir().join("ferrishot.log"),
    )
});
