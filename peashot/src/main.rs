//! The ferrishot app

use std::sync::Arc;

use clap::Parser as _;
use ferrishot::Cli;
use miette::IntoDiagnostic as _;
use miette::miette;

use ferrishot::App;
use tap::Pipe as _;

/// RGBA bytes for the Logo of ferrishot. Generated with `build.rs`
const LOGO: &[u8; 64 * 64 * 4] = include_bytes!(concat!(env!("OUT_DIR"), "/logo.bin"));

#[allow(
    clippy::print_stderr,
    clippy::print_stdout,
    reason = "print from `main` is fine"
)]
fn main() -> miette::Result<()> {
    // On linux, a daemon is required to provide clipboard access even when
    // the process dies.
    //
    // If no daemon then:
    // - Something is copied to the clipboard. It can be pasted into other apps just fine.
    // - But, if the process from which the thing was copied dies: so does whatever we copied.
    //   Clipboard empties!
    //
    // This daemon is invoked by ferrishot itself. We spawn a new `ferrishot` process and
    // pass in a unique argument to ourselves.
    //
    // If we receive this argument we become a daemon, running a background process
    // instead of the usual screenshot app which provides the clipboard item until the
    // user copies something else to their clipboard.
    //
    // More info: <https://docs.rs/arboard/3.5.0/arboard/trait.SetExtLinux.html#tymethod.wait>
    #[cfg(target_os = "linux")]
    if std::env::args().nth(1).as_deref() == Some(ferrishot::CLIPBOARD_DAEMON_ID) {
        ferrishot::run_clipboard_daemon().expect("Failed to run clipboard daemon");
        return Ok(());
    }

    // Parse command line arguments
    let cli = Arc::new(Cli::parse());

    // Setup logging
    ferrishot::logging::initialize(&cli);

    if cli.dump_default_config {
        std::fs::create_dir_all(
            std::path::PathBuf::from(&cli.config_file)
                .parent()
                .ok_or_else(|| miette!("Could not get parent path of {}", cli.config_file))?,
        )
        .into_diagnostic()?;

        std::fs::write(&cli.config_file, ferrishot::DEFAULT_KDL_CONFIG_STR).into_diagnostic()?;

        if !cli.silent {
            println!("Wrote the default config file to {}", cli.config_file);
        }

        return Ok(());
    }

    // these variables need to be re-used after the `iced::application` ends
    let cli_save_path = cli.save_path.clone();
    let is_silent = cli.silent;

    if let Some(delay) = cli.delay {
        if !cli.silent {
            println!("Sleeping for {delay:?}...");
        }
        std::thread::sleep(delay);
    }

    // Parse user's `ferrishot.kdl` config file
    let config = Arc::new(ferrishot::Config::parse(&cli.config_file)?);

    // The image that we are going to be editing
    let image = Arc::new(ferrishot::get_image(cli.file.as_ref())?);

    // start the app with an initial selection of the image
    let initial_region = if cli.last_region {
        ferrishot::last_region::read(image.bounds())?
    } else {
        cli.region.map(|lazy_rect| lazy_rect.init(image.bounds()))
    };

    let generate_output = match (cli.accept_on_select, initial_region) {
        // If we want to do an action as soon as we have a selection,
        // AND we start the app with the selection: Then don't even launch a window.
        //
        // Run in 'headless' mode and perform the action instantly
        (Some(accept_on_select), Some(region)) => {
            let runtime = tokio::runtime::Runtime::new().into_diagnostic()?;

            App::headless(accept_on_select, region, image, cli.json)
                .pipe(|fut| runtime.block_on(fut))
                .map_err(|err| miette!("Failed to start ferrishot (headless): {err}"))?
                .pipe(Some)
        }
        // Launch full ferrishot app
        _ => {
            iced::application(
                move || {
                    App::builder()
                        .cli(Arc::clone(&cli))
                        .config(Arc::clone(&config))
                        .maybe_initial_region(initial_region)
                        .image(Arc::clone(&image))
                        .build()
                },
                App::update,
                App::view,
            )
            .subscription(App::subscription)
            .window(iced::window::Settings {
                level: iced::window::Level::Normal,
                fullscreen: true,
                icon: Some(
                    iced::window::icon::from_rgba(LOGO.to_vec(), 64, 64)
                        .expect("Icon to be valid RGBA bytes"),
                ),
                ..Default::default()
            })
            .title("ferrishot")
            .default_font(iced::Font::MONOSPACE)
            .run()
            .map_err(|err| miette!("Failed to start ferrishot: {err}"))?;

            None
        }
    };

    let saved_path = if let Some(saved_image) = ferrishot::SAVED_IMAGE.get() {
        if let Some(save_path) = cli_save_path.or_else(|| {
            // Open file explorer to choose where to save the image
            let dialog = rfd::FileDialog::new()
                .set_title("Save Screenshot")
                .save_file();

            if dialog.is_none() {
                log::info!("The file dialog was closed before a file was chosen");
            }

            dialog
        }) {
            saved_image
                .save(&save_path)
                .map_err(|err| miette!("Failed to save the screenshot: {err}"))?;

            Some(save_path)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(print_output) = generate_output {
        let output = print_output(saved_path);
        if !is_silent {
            print!("{output}");
        }
    }
    Ok(())
}
