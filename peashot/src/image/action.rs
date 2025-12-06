//! One of 3 actions:
//!
//! - Upload image
//! - Copy image
//! - Save image
use std::path::PathBuf;

use iced::Rectangle;
use iced::Task;
use image::DynamicImage;

use crate::image::upload::ImageUploaded;
use crate::{App, geometry::RectangleExt as _, ui::popup::image_uploaded};
use iced::widget;

// INFO: Documentation comments for the enum are used in `--help`
crate::declare_commands! {
    #[derive(clap::ValueEnum)]
    /// Action to take with the image
    enum Command {
        /// Copy image to the clipboard
        UploadScreenshot,
        /// Save image to a file
        CopyToClipboard,
        /// Upload image to the internet
        SaveScreenshot,
    }
}

impl crate::command::Handler for Command {
    fn handle(self, app: &mut App, _count: u32) -> Task<crate::Message> {
        let Some(rect) = app.selection.map(|sel| sel.rect.norm()) else {
            app.errors.push(match self {
                Self::CopyToClipboard => "There is no selection to copy",
                Self::UploadScreenshot => "There is no selection to upload",
                Self::SaveScreenshot => "There is no selection to save",
            });
            return Task::none();
        };

        if self == Self::UploadScreenshot {
            app.is_uploading_image = true;
        }

        let image = App::process_image(rect, &app.image);

        Task::future(async move {
            match self.execute(image, rect).await {
                Ok((Output::Saved | Output::Copied, _)) => crate::message::Message::Exit,
                Ok((
                    Output::Uploaded {
                        path,
                        data,
                        file_size,
                    },
                    ImageData { height, width },
                )) => crate::Message::ImageUploaded(image_uploaded::Message::ImageUploaded(
                    image_uploaded::ImageUploadedData {
                        image_uploaded: data,
                        uploaded_image: widget::image::Handle::from_path(&path),
                        height,
                        width,
                        file_size,
                    },
                )),
                Err(err) => crate::Message::Error(err.to_string()),
            }
        })
    }
}

/// Data about the image
pub struct ImageData {
    /// Height of the image (pixels)
    pub height: u32,
    /// Width of the image (pixels)
    pub width: u32,
}

/// The output of an image action
pub enum Output {
    /// Copied to the clipboard
    Copied,
    /// Saved to a path
    ///
    /// We don't know the path yet. We'll find out at the end of `main`.
    Saved,
    /// Uploaded to the internet
    Uploaded {
        /// information about the uploaded image
        data: ImageUploaded,
        /// file size in bytes
        file_size: u64,
        /// Path to the uploaded image
        path: PathBuf,
    },
}

/// Image action error
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    /// Clipboard error
    #[error("failed to copy the image: {0}")]
    Clipboard(#[from] crate::clipboard::ClipboardError),
    /// IO error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Image upload error
    #[error("failed to upload the image: {0}")]
    ImageUpload(String),
    /// Image error
    #[error(transparent)]
    SaveImage(#[from] image::ImageError),
    /// Could not get the image
    #[error(transparent)]
    GetImage(#[from] crate::image::GetImageError),
}

impl Command {
    /// Convert this into a key action
    pub fn into_key_action(self) -> crate::Command {
        match self {
            Self::CopyToClipboard => crate::Command::ImageUpload(Self::CopyToClipboard),
            Self::SaveScreenshot => crate::Command::ImageUpload(Self::SaveScreenshot),
            Self::UploadScreenshot => crate::Command::ImageUpload(Self::UploadScreenshot),
        }
    }

    /// Execute the action
    pub async fn execute(
        self,
        image: DynamicImage,
        region: Rectangle,
    ) -> Result<(Output, ImageData), Error> {
        let image_data = ImageData {
            height: image.height(),
            width: image.width(),
        };

        // NOTE: Not a hard error, so no need to abort the main action
        if let Err(failed_to_write) = crate::last_region::write(region) {
            log::error!(
                "Failed to save the current rectangle selection, for possible re-use: {failed_to_write}"
            );
        }

        let out = match self {
            Self::CopyToClipboard => crate::clipboard::set_image(arboard::ImageData {
                width: image.width() as usize,
                height: image.height() as usize,
                bytes: std::borrow::Cow::Borrowed(image.as_bytes()),
            })
            .map(|_| (Output::Copied, image_data))?,
            Self::SaveScreenshot => {
                let _ = SAVED_IMAGE.set(image);
                (Output::Saved, image_data)
            }
            Self::UploadScreenshot => {
                let path = tempfile::TempDir::new()?
                    .into_path()
                    .join("ferrishot-screenshot.png");

                // TODO: allow configuring the upload format
                // in-app
                image.save_with_format(&path, image::ImageFormat::Png)?;

                (
                    Output::Uploaded {
                        data: crate::image::upload::upload(&path).await.map_err(|err| {
                            err.into_iter()
                                .next()
                                .map(Error::ImageUpload)
                                .expect("at least 1 image upload provider")
                        })?,
                        file_size: path.metadata().map(|meta| meta.len()).unwrap_or(0),
                        path,
                    },
                    image_data,
                )
            }
        };

        Ok(out)
    }
}

/// The image to save to a file, chosen by the user in a file picker.
///
/// Unfortunately, there is simply no way to communicate something from
/// the inside of an iced application to the outside: i.e. "Return" something
/// from an iced program exiting. So we have to use a global variable for this.
///
/// This global is mutated just *once* at the end of the application's lifetime,
/// when the window closes.
///
/// It is then accessed just *once* to open the file dialog and let the user pick
/// where they want to save their image.
///
/// Yes, at the moment we want this when using Ctrl + S to save as file:
/// 1. Close the application to save the file and generate the image we'll save
/// 2. Open the file explorer, and save the image to the specified path
///
/// When the file explorer is spawned from the inside of an iced window, closing
/// this window will then also close the file explorer. It means that we can't
/// close the window and then spawn an explorer.
///
/// The other option is to have both windows open at the same time. But this
/// would be really odd. First of all, we will need to un-fullscreen the App
/// because the file explorer can spawn under the app.
///
/// This is going to be sub-optimal. Currently, we give off the illusion of
/// drawing shapes and rectangles on top of the desktop. It is not immediately
/// obvious that the app is just another window which is full-screen.
/// Doing the above would break that illusion.
///
/// Ideally, we would be able to spawn a file explorer *above* the window without
/// having to close this. But this seems to not be possible. Perhaps in the
/// future there will be some kind of file explorer Iced widget that we
/// can use instead of the native file explorer.
pub static SAVED_IMAGE: std::sync::OnceLock<DynamicImage> = std::sync::OnceLock::new();
