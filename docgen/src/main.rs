//! Generate completions, docs, etc
use std::{fs::File, io::Write, path::PathBuf};

use clap::{CommandFactory, ValueEnum};
use clap_complete::generate_to;
use clap_markdown::MarkdownOptions;
use peashot::Cli;

fn main() {
    let mut cmd = Cli::command();
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("completions");

    std::fs::create_dir_all(&out_dir).unwrap();

    // shell completions
    for shell in clap_complete::Shell::value_variants() {
        generate_to(*shell, &mut cmd, "ferrishot", &out_dir).unwrap();
    }
    generate_to(
        clap_complete_nushell::Nushell,
        &mut cmd,
        "ferrishot",
        &out_dir,
    )
    .unwrap();
    generate_to(carapace_spec_clap::Spec, &mut cmd, "ferrishot", &out_dir).unwrap();
    generate_to(clap_complete_fig::Fig, &mut cmd, "ferrishot", &out_dir).unwrap();

    // markdown help
    File::create(out_dir.join("ferrishot.md"))
        .unwrap()
        .write_all(
            clap_markdown::help_markdown_custom::<Cli>(&MarkdownOptions::new().show_footer(false))
                .as_bytes(),
        )
        .unwrap();

    // man page
    clap_mangen::generate_to(Cli::command(), out_dir).unwrap();
}
