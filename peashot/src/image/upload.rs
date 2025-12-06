//! Upload images to free services

use std::path::Path;

use ferrishot_knus::DecodeScalar;
use iced::futures::future::join_all;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use strum::{EnumCount as _, IntoEnumIterator as _};
use tokio::sync::oneshot;

/// A single client for HTTP requests
static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

/// Upload an image to multiple services. As soon as the first service succeeds,
/// cancel the other uploads.
///
/// # Returns
///
/// Link to the uploaded image
///
/// # Errors
///
/// If none succeed, return error for all the services
pub async fn upload(file_path: &Path) -> Result<ImageUploaded, Vec<String>> {
    let mut handles = Vec::new();

    // Channel for results
    // Each uploader sends either Ok(url) or Err(err), tagged with index of the uploader
    let (tx, mut rx) =
        tokio::sync::mpsc::unbounded_channel::<(usize, Result<ImageUploaded, String>)>();

    // Channel for cancellation
    // Sending end `cancel_tx` triggered by the first successful uploader
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    let cancel_rx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(cancel_rx)));

    // launch an Upload task for each service
    for (i, service) in ImageUploadService::iter().enumerate() {
        let tx = tx.clone();
        let path = file_path.to_path_buf();
        let cancel_rx = cancel_rx.clone();

        handles.push(tokio::spawn(async move {
            let cancel = {
                let mut rx_lock = cancel_rx.lock().await;
                rx_lock.take()
            };

            tokio::select! {
                biased;

                () = async {
                    if let Some(rx) = cancel {
                        let _ = rx.await;
                    }
                } => {
                    // cancelled, do nothing
                }

                response = service.upload_image(&path) => {
                    let result = response.map_err(|e| e.to_string());
                    let _ = tx.send((i, result));
                }
            };
        }));
    }

    // receiver stops waiting if no senders remain
    drop(tx);

    let mut errors = vec![None; ImageUploadService::COUNT];

    while let Some((i, result)) = rx.recv().await {
        match result {
            Ok(url) => {
                // cancel other Upload tasks
                let _ = cancel_tx.send(());

                join_all(handles).await;
                return Ok(url);
            }
            Err(err) => {
                errors[i] = Some(err);
            }
        }

        if errors.iter().all(Option::is_some) {
            break;
        }
    }

    join_all(handles).await;

    Err(errors.into_iter().flatten().collect())
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Debug,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    DecodeScalar,
    strum::EnumIter,
    strum::EnumCount,
)]
#[serde(rename_all = "kebab-case")]
/// Choose which image upload service should be used by default when pressing "Upload Online"
pub enum ImageUploadService {
    /// - Website: `https://litterbox.catbox.moe`
    /// - Max upload size: 1 GB
    Litterbox,
    /// - Website: `https://catbox.moe`
    /// - Max upload size: 200 MB
    Catbox,
    /// - Website: `https://0x0.st`
    /// - Max upload size: 512 MiB
    TheNullPointer,
    /// - Website: `https://uguu.se`
    /// - Max upload size: 128 Mib
    Uguu,
}

/// Data of the uploaded image
#[derive(Debug, Clone)]
pub struct ImageUploaded {
    /// Link to the uploaded image
    pub link: String,
    /// How long until the image expires (rough estimate - purely for visualization)
    pub expires_in: &'static str,
}

/// Image upload error
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    /// IO error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Invalid response. serde could not parse
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

impl ImageUploadService {
    /// Conservative estimate for how long until images expire
    fn expires_in(self) -> &'static str {
        match self {
            Self::Litterbox => "3 days",
            Self::Catbox => "2 weeks",
            Self::TheNullPointer => "30 days",
            Self::Uguu => "3 hours",
        }
    }

    /// The base URL where image files should be uploaded
    fn post_url(self) -> &'static str {
        match self {
            Self::TheNullPointer => "https://0x0.st",
            Self::Uguu => "https://uguu.se/upload",
            Self::Catbox => "https://catbox.moe/user/api.php",
            Self::Litterbox => "https://litterbox.catbox.moe/resources/internals/api.php",
        }
    }

    /// Upload the image to the given upload service
    pub async fn upload_image(self, file_path: &Path) -> Result<ImageUploaded, Error> {
        let request = HTTP_CLIENT
            .request(reqwest::Method::POST, self.post_url())
            .header(
                "User-Agent",
                format!("ferrishot/{:?}", env!("CARGO_PKG_VERSION")),
            );

        let link = match self {
            Self::TheNullPointer => {
                request
                    .multipart(Form::new().file("file", file_path).await?)
                    .send()
                    .await?
                    .text()
                    .await?
            }
            Self::Uguu => {
                #[derive(Serialize, Deserialize)]
                struct UguuResponse {
                    /// Array of uploaded files
                    files: Vec<UguuFile>,
                }

                #[derive(Serialize, Deserialize)]
                struct UguuFile {
                    /// Link to the uploaded image
                    url: String,
                }

                request
                    .multipart(Form::new().file("files[]", file_path).await?)
                    .send()
                    .await?
                    .json::<UguuResponse>()
                    .await?
                    .files
                    .into_iter()
                    .next()
                    .ok_or(Error::InvalidResponse(
                        "Expected uguu to return an array with 1 file".to_string(),
                    ))?
                    .url
            }
            Self::Catbox => {
                request
                    .multipart(
                        Form::new()
                            .part("reqtype", Part::text("fileupload"))
                            .file("fileToUpload", file_path)
                            .await?,
                    )
                    .send()
                    .await?
                    .text()
                    .await?
            }
            Self::Litterbox => {
                request
                    .multipart(
                        Form::new()
                            .part("reqtype", Part::text("fileupload"))
                            .part("time", Part::text("72h"))
                            .file("fileToUpload", file_path)
                            .await?,
                    )
                    .send()
                    .await?
                    .text()
                    .await?
            }
        };

        Ok(ImageUploaded {
            link,
            expires_in: self.expires_in(),
        })
    }
}
