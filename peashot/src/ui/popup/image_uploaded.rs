//! Show a popup when the image has already been uploaded
//!
//! Popup contains:
//!
//! - QR Code
//! - Copy URL to clipboard
//! - Image metadata
//! - Image preview

use std::{thread, time::Duration};

use iced::{
    Background, Element,
    Length::{self, Fill},
    Size, Task,
    widget::{button, column, container, horizontal_rule, qr_code, row, svg, text, tooltip},
};

use crate::icon;

use super::Popup;
use crate::ui::selection_icons::icon_tooltip;

/// State for the uploaded image popup
#[derive(Debug)]
pub struct State {
    /// A link to the uploaded image
    pub url: (qr_code::Data, ImageUploadedData),
    /// When clicking on "Copy" button, change it to be a green tick for a few seconds before
    /// reverting back
    pub has_copied_link: bool,
}

/// Message for the image uploaded
#[derive(Clone, Debug)]
pub enum Message {
    /// The image was uploaded to the internet
    ImageUploaded(ImageUploadedData),
    /// Copy link of image to clipboard
    CopyLink(String),
    /// Some time has passed after the link was copied
    CopyLinkTimeout,
}

impl crate::message::Handler for Message {
    fn handle(self, app: &mut crate::App) -> Task<crate::Message> {
        match self {
            Self::CopyLinkTimeout => {
                if let Some(image_uploaded) = app
                    .popup
                    .as_mut()
                    .and_then(|p| p.try_as_image_uploaded_mut())
                {
                    image_uploaded.has_copied_link = false;
                }
            }
            Self::CopyLink(url) => {
                if let Err(err) = crate::clipboard::set_text(&url) {
                    app.errors.push(err.to_string());
                } else {
                    if let Some(image_uploaded) = app
                        .popup
                        .as_mut()
                        .and_then(|p| p.try_as_image_uploaded_mut())
                    {
                        image_uploaded.has_copied_link = true;
                    }
                    return Task::future(async move {
                        thread::sleep(Duration::from_secs(3));
                        crate::Message::ImageUploaded(Self::CopyLinkTimeout)
                    });
                }
            }
            Self::ImageUploaded(data) => {
                app.is_uploading_image = false;
                match qr_code::Data::new(data.image_uploaded.link.clone()) {
                    Ok(qr_code) => {
                        app.popup = Some(Popup::ImageUploaded(State {
                            url: (qr_code, data),
                            has_copied_link: false,
                        }));
                        app.selection = None;
                    }
                    Err(err) => {
                        app.errors.push(format!("Failed to get QR Code: {err}"));
                    }
                }
            }
        }

        Task::none()
    }
}

/// Data of the uploaded image
#[derive(Clone, Debug)]
pub struct ImageUploadedData {
    /// data of the image upload
    pub image_uploaded: crate::image::upload::ImageUploaded,
    /// the uploaded image
    pub uploaded_image: iced::widget::image::Handle,
    /// The height of the image
    pub height: u32,
    /// The width of the image
    pub width: u32,
    /// File size in bytes
    pub file_size: u64,
}

/// Data for the uploaded image
pub struct ImageUploaded<'app> {
    /// The App
    pub app: &'app crate::App,
    /// Data for the URL to the uploaded image
    pub qr_code_data: &'app qr_code::Data,
    /// When the URL Was copied
    pub url_copied: bool,
    /// Data of the uploaded image
    pub data: &'app ImageUploadedData,
}

impl<'app> ImageUploaded<'app> {
    /// Render the QR Code
    pub fn view(&self) -> Element<'app, crate::Message> {
        let size = Size::new(700.0, 1200.0);
        super::popup(
            size,
            container(
                column![
                    //
                    // Heading
                    //
                    container(text("Image Uploaded").size(30.0)).center_x(Fill),
                    //
                    // Divider
                    //
                    container(horizontal_rule(2)).height(10.0),
                    //
                    // URL Text + Copy Button + QR Code
                    //
                    container(
                        column![
                            //
                            // URL Text + Copy Button
                            //
                            container(row![
                                //
                                // URL Text
                                //
                                container(
                                    text(self.data.image_uploaded.link.clone())
                                        .color(self.app.config.theme.image_uploaded_fg)
                                )
                                .center_y(Fill),
                                //
                                // Copy to clipboard button
                                //
                                {
                                    let (clipboard_icon, clipboard_icon_color, label) =
                                        if self.url_copied {
                                            (icon!(Check), self.app.config.theme.success, "Copied!")
                                        } else {
                                            (
                                                icon!(Clipboard),
                                                self.app.config.theme.image_uploaded_fg,
                                                "Copy Link",
                                            )
                                        };

                                    container(icon_tooltip(
                                        button(
                                            clipboard_icon
                                                .style(move |_, _| svg::Style {
                                                    color: Some(clipboard_icon_color),
                                                })
                                                .width(Length::Fixed(25.0))
                                                .height(Length::Fixed(25.0)),
                                        )
                                        .on_press(crate::Message::ImageUploaded(Message::CopyLink(
                                            self.data.image_uploaded.link.to_string(),
                                        )))
                                        .style(|_, _| {
                                            button::Style {
                                                background: Some(Background::Color(
                                                    iced::Color::TRANSPARENT,
                                                )),
                                                ..Default::default()
                                            }
                                        }),
                                        text(label),
                                        tooltip::Position::Top,
                                        &self.app.config.theme,
                                    ))
                                    .center_y(Fill)
                                }
                            ])
                            .style(|_| container::Style {
                                text_color: Some(self.app.config.theme.image_uploaded_fg),
                                ..Default::default()
                            })
                            .center_y(Length::Fixed(32.0))
                            .center_x(Fill),
                            //
                            // QR Code
                            //
                            container(qr_code(self.qr_code_data).total_size(250.0)).center_x(Fill),
                        ]
                        .spacing(30.0)
                    )
                    .center(Fill)
                    .height(Length::Fixed(300.0)),
                    //
                    // --- Preview ---
                    //
                    container(
                        row![
                            container(horizontal_rule(2)).center_y(Fill),
                            container(text("Preview").size(30.0)).center_y(Fill),
                            container(horizontal_rule(2)).center_y(Fill)
                        ]
                        .spacing(20.0)
                    )
                    .center_x(Fill),
                    //
                    // Metadata
                    //
                    container(column![
                        text!(
                            "Image dimensions: {w} ✕ {h}",
                            w = self.data.width,
                            h = self.data.height
                        )
                        .shaping(text::Shaping::Advanced),
                        text!(
                            "Filesize: {}",
                            human_bytes::human_bytes(self.data.file_size as f64)
                        ),
                        text!("Link expires in: {}", self.data.image_uploaded.expires_in)
                    ])
                    .center_x(Fill),
                    //
                    // Image
                    //
                    iced::widget::image(self.data.uploaded_image.clone()).width(Fill)
                ]
                .spacing(30.0),
            )
            .width(size.width)
            .height(size.height)
            .style(|_| container::Style {
                text_color: Some(self.app.config.theme.image_uploaded_fg),
                background: Some(Background::Color(self.app.config.theme.image_uploaded_bg)),
                ..Default::default()
            })
            .padding(30.0),
            &self.app.config.theme,
        )
    }
}
