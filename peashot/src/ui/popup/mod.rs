//! Popups are overlaid on top of the screen.
//! They block any inputs
//!
//! Only one of the popups can be active at any time (see `Popup` enum)

pub mod keybindings_cheatsheet;
use iced::Background;
use iced::Element;
use iced::Length::Fill;
pub use keybindings_cheatsheet::KeybindingsCheatsheet;

pub mod image_uploaded;
pub use image_uploaded::ImageUploaded;

use iced::widget::{
    button, column, container, horizontal_space, row, stack, svg, tooltip, vertical_space,
};
pub mod letters;
pub use letters::Letters;

/// Popup are overlaid on top and they block any events. allowing only Escape to close
/// the popup.
#[derive(Debug, strum::EnumTryAs)]
pub enum Popup {
    /// Letters allow picking a one of 10,000+ regions on the screen in 4 keystrokes
    Letters(letters::State),
    /// An image has been uploaded to the internet
    ImageUploaded(image_uploaded::State),
    /// Shows available commands
    KeyCheatsheet,
}

/// Elements inside of a `popup` render in the center of the screen
/// with a close button
fn popup<'app>(
    size: iced::Size,
    contents: impl Into<Element<'app, crate::Message>>,
    theme: &'app crate::Theme,
) -> Element<'app, crate::Message> {
    container(stack![
        contents.into(),
        //
        // Close Button 'x' in the top right corner
        //
        column![
            vertical_space().height(10.0),
            row![
                horizontal_space().width(Fill),
                super::selection_icons::icon_tooltip(
                    button(
                        crate::icon!(Close)
                            .style(|_, _| svg::Style {
                                color: Some(theme.popup_close_icon_fg)
                            })
                            .width(24.0)
                            .height(24.0)
                    )
                    .on_press(crate::Message::ClosePopup)
                    .style(|_, _| button::Style {
                        background: Some(Background::Color(theme.popup_close_icon_bg)),
                        ..Default::default()
                    }),
                    "Close",
                    tooltip::Position::Right,
                    theme
                ),
                horizontal_space().width(10.0)
            ]
            .height(size.height)
            .width(size.width)
        ]
    ])
    .center(Fill)
    .into()
}
