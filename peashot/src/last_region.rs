//! Read and write the last region of a rectangle
use crate::{
    geometry::RectangleExt as _,
    lazy_rect::{LazyRectangle, ParseRectError},
};
use etcetera::BaseStrategy as _;
use iced::Rectangle;
use std::{fs, io::Write as _, str::FromStr as _};
use tap::Pipe as _;

/// Could not get the last region
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    /// Can't find home dir
    #[error(transparent)]
    HomeDir(#[from] etcetera::HomeDirError),
    /// Failed to parse as rectangle
    #[error(transparent)]
    ParseRect(#[from] ParseRectError),
    /// Failed to read the last region file
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
/// Name of the file used to read the last region
pub const LAST_REGION_FILENAME: &str = "ferrishot-last-region.txt";

/// Read the last region used
pub fn read(image_bounds: Rectangle) -> Result<Option<Rectangle>, Error> {
    etcetera::choose_base_strategy()?
        .cache_dir()
        .join(LAST_REGION_FILENAME)
        .pipe(fs::read_to_string)?
        .pipe_deref(LazyRectangle::from_str)?
        .pipe(|lazy_rect| lazy_rect.init(image_bounds))
        .pipe(Some)
        .pipe(Ok)
}

/// Write the last used region
pub(crate) fn write(region: Rectangle) -> Result<(), Error> {
    etcetera::choose_base_strategy()?
        .cache_dir()
        .join(LAST_REGION_FILENAME)
        .pipe(fs::File::create)?
        .write_all(region.as_str().as_bytes())?
        .pipe(Ok)
}

#[cfg(not(target_os = "linux"))]
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn write_and_read_last_region() {
        let region = Rectangle {
            x: 42.0,
            y: 24.0,
            width: 800.0,
            height: 600.0,
        };

        write(region).unwrap();
        assert_eq!(
            read(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 3440.0,
                height: 1440.00
            })
            .unwrap(),
            Some(region)
        );
        let another_region = Rectangle {
            x: 900.0,
            y: 400.0,
            width: 800.0,
            height: 150.0,
        };

        write(another_region).unwrap();
        assert_eq!(
            read(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 3440.0,
                height: 1440.00
            })
            .unwrap(),
            Some(another_region)
        );
    }
}
