//! Tests for ferrishot

// #![allow(clippy::unwrap_used, reason = "relaxed rules in tests")]
// //! Tests

// use std::path::PathBuf;

// use clap::Parser as _;
// use ferrishot::Cli;
// use tempfile::NamedTempFile;

// fn cli(args: &[&str]) -> (Cli, PathBuf) {
//     let temp_path = NamedTempFile::new().unwrap();
//     let cli = Cli::parse_from(
//         vec![
//             "--accept-on-select",
//             "save-screenshot",
//             "--save-path",
//             temp_path.path().to_str().unwrap(),
//         ]
//         .iter()
//         .chain(args),
//     );

//     (cli, temp_path.path().to_path_buf())
// }

// /// Help and version must output to the standard output
// ///
// /// They need to
// #[test]
// fn help_and_version() {
//     let (cli, saved_path) = cli(&["-V"]);
//     assert_eq!(saved_path)
// }

// #[test]
// fn region_100x100_bottom_right_corner() {
//     let cli = Cli::parse_from(vec![
//         "--accept-on-select",
//         "save-screenshot",
//         "--save-path",
//         temp.path().as_os_str().to_str().unwrap(),
//         "--region",
//     ]);
// }
