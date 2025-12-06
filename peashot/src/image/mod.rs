//! Contains ways of opening the image / uploading / saving / copying it

pub mod action;

pub mod upload;

mod screenshot;
use std::path::PathBuf;

use image::ImageReader;

mod rgba_handle;
pub use rgba_handle::RgbaHandle;
use tap::Pipe as _;

/// Failed to get the image
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum GetImageError {
    /// IO error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Image error
    #[error(transparent)]
    Image(#[from] image::ImageError),
    /// Screenshot error
    #[error(transparent)]
    Screenshot(#[from] screenshot::ScreenshotError),
}

/// Returns handle of the image that will be edited
///
/// If path is passed, use that as the image to edit.
/// Otherwise take a screenshot of the desktop and use that to edit.
pub fn get_image(file: Option<&PathBuf>) -> Result<RgbaHandle, GetImageError> {
    file.map(ImageReader::open)
        .transpose()?
        .map(ImageReader::decode)
        .transpose()?
        .map_or_else(
            // no path passed = take image of the monitor
            screenshot::take,
            |img| RgbaHandle::new(img.width(), img.height(), img.into_rgba8().into_raw()).pipe(Ok),
        )?
        .pipe(Ok)
}
