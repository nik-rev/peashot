//! The welcome message contains tips on how to use ferrishot

use iced::{
    Background, Element, Font,
    Length::{self, Fill},
    alignment::Vertical,
    widget::{Column, Space, column, row, text, text::Shaping},
};

use crate::message::Message;

/// Width of the welcome message box
const WIDTH: u32 = 380;
/// Size of the font in the welcome message box
const FONT_SIZE: f32 = 13.0;
/// Tips to show to the user in the welcome message
const SPACING: f32 = 8.0;
/// Padding of the tips
const PADDING: f32 = 10.0;
/// Tips: The Key, and Action for each Key
const TIPS: [(&str, &str); 7] = [
    ("Mouse", "Select screenshot area"),
    ("Ctrl + S", "Save screenshot to a file"),
    ("Enter", "Copy screenshot to clipboard"),
    ("Right Click", "Snap closest corner to mouse"),
    ("Shift + Mouse", "Slowly resize / move area"),
    ("?", "Open Keybindings Cheatsheet"),
    ("Esc", "Exit"),
];
/// Height of the welcome message box
const HEIGHT: f32 =
    30.0 + TIPS.len() as f32 * FONT_SIZE + (TIPS.len() - 1) as f32 * SPACING + (PADDING * 2.0);

/// Renders the welcome message that the user sees when they first launch the program
pub fn welcome_message(app: &super::App) -> Element<Message> {
    let image_width = app.image.width();
    let image_height = app.image.height();
    let vertical_space = Space::with_height(image_height / 2 - HEIGHT as u32 / 2);
    let horizontal_space = Space::with_width(image_width / 2 - WIDTH / 2);

    let stuff = iced::widget::container(
        TIPS.into_iter()
            .map(|(key, action)| {
                row![
                    row![
                        Space::with_width(Fill),
                        text(key)
                            .size(FONT_SIZE)
                            .font(Font {
                                weight: iced::font::Weight::Bold,
                                ..Font::default()
                            })
                            .shaping(Shaping::Advanced)
                            .align_y(Vertical::Bottom)
                    ]
                    .width(100.0),
                    Space::with_width(Length::Fixed(20.0)),
                    text(action).size(FONT_SIZE).align_y(Vertical::Bottom),
                ]
                .into()
            })
            .collect::<Column<_>>()
            .spacing(SPACING)
            .height(HEIGHT)
            .width(WIDTH)
            .padding(PADDING),
    )
    .style(|_| iced::widget::container::Style {
        text_color: Some(app.config.theme.info_box_fg),
        background: Some(Background::Color(app.config.theme.info_box_bg)),
        border: iced::Border::default()
            .color(app.config.theme.info_box_border)
            .rounded(6.0)
            .width(1.5),
        shadow: iced::Shadow::default(),
    });

    column![vertical_space, row![horizontal_space, stuff]].into()
}
