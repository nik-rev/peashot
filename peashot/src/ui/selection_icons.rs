//! Icons around the selection rectangle

use core::f32;

use iced::{
    Element,
    Length::{self, Fill},
    Padding, Radians, Rectangle, Rotation,
    widget::{Column, Row, Space, row, tooltip},
};

use crate::{icon, message::Message, ui::selection::FRAME_WIDTH};
use crate::{lazy_rect::LazyRectangle, ui::selection::ICON_BUTTON_SIZE};
use iced::{Background, Border, Shadow, widget};

use super::app::{self};

// Here is the behaviour that we want
//
// We have a list of icons we want to render.
// We want to render every single one of them.
// Each icon should not be shrunk, nor should it render in weird positions
//
// for each side in [bottom, right, top, left] we render
// all of the icons that fit on that side.
//
// But then we may have a small selection which doesn't manage to render all of the icons,
// so we deal with that by rendering a couple extra rows on top and bottom

/// Height and width of each icon
const PX_PER_ICON: f32 = SPACE_BETWEEN_ICONS + ICON_BUTTON_SIZE;
/// The minimum amount of icons to render at the top
const MIN_TOP_BOTTOM_ICONS: usize = 3;
/// The minimum amount of icons to render on the sides
const MIN_SIDE_ICONS: usize = 1;
/// Space in-between each icon
const SPACE_BETWEEN_ICONS: f32 = 2.0;

/// Create a tooltip for an icon
pub fn icon_tooltip<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    tooltip: impl Into<Element<'a, Message>>,
    position: tooltip::Position,
    theme: &'a crate::Theme,
) -> widget::Tooltip<'a, Message> {
    widget::Tooltip::new(content, tooltip, position)
        .style(move |_| widget::container::Style {
            text_color: Some(theme.tooltip_fg),
            background: Some(Background::Color(theme.tooltip_bg)),
            border: Border::default(),
            shadow: Shadow::default(),
        })
        .gap(10.0)
}

/// Styled icon as a button
pub fn selection_icon<'a, Message>(
    icon: widget::Svg<'a>,
    theme: &'a crate::Theme,
) -> widget::Button<'a, Message> {
    /// Width and height for icons *inside* of buttons
    const ICON_SIZE: f32 = 32.0;

    widget::button(
        icon.style(move |_, _| widget::svg::Style {
            color: Some(theme.icon_fg),
        })
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE)),
    )
    .width(Length::Fixed(ICON_BUTTON_SIZE))
    .height(Length::Fixed(ICON_BUTTON_SIZE))
    .style(move |_, _| {
        let mut style = widget::button::Style::default().with_background(theme.icon_bg);
        style.shadow = Shadow {
            color: theme.drop_shadow,
            blur_radius: 3.0,
            offset: iced::Vector { x: 0.0, y: 0.0 },
        };
        style.border =
            Border::default().rounded(iced::border::Radius::new(iced::Pixels::from(f32::INFINITY)));
        style
    })
}

/// Icons around the selection
#[derive(Debug, Copy, Clone)]
pub struct SelectionIcons<'app> {
    /// The App
    pub app: &'app super::App,
    /// Width of the container which contains `inner_rect`
    pub image_width: f32,
    /// Height of the container which contains `inner_rect`
    pub image_height: f32,
    /// Rectangle around which the icons will render.
    /// Represents the `Selection`'s rectangle
    pub selection_rect: Rectangle,
}

/// Add icons to the side until the amount of them reaches the minimum required
fn add_icons_until_there_is_at_least_n_of_them<'a, const MIN_ELEMENTS: usize>(
    mut icons: Vec<Element<'a, Message>>,
    mut iter: impl Iterator<Item = (Element<'a, Message>, &'static str)>,
    mut padding: f32,
    total_icons_positioned: &mut usize,
    tooltip_position: tooltip::Position,
    theme: &'a crate::Theme,
) -> (Vec<Element<'a, Message>>, f32) {
    while icons.len() < MIN_ELEMENTS {
        if let Some((next, tooltip_str)) = iter.by_ref().next() {
            icons.push(icon_tooltip(next, tooltip_str, tooltip_position, theme).into());
            *total_icons_positioned += 1;
            padding -= PX_PER_ICON / 2.0;
        } else {
            break;
        }
    }
    (icons, padding)
}

/// Position icons until we reach an adequate amount of them
fn position_icons_in_line<'a>(
    space_available: f32,
    tooltip_position: tooltip::Position,
    total_icons_positioned: &mut usize,
    mut icons_iter: impl Iterator<Item = (Element<'a, Message>, &'static str)>,
    icons_len: usize,
    theme: &'a crate::Theme,
) -> (Vec<Element<'a, Message>>, f32) {
    let icons_left_to_position = icons_len - *total_icons_positioned;
    let icons_rendered_here =
        ((space_available / PX_PER_ICON) as usize).min(icons_left_to_position);
    *total_icons_positioned += icons_rendered_here;

    // we do this thing because we need to know exactly
    // how many elems we got. size_hint may be unreliable
    let mut icons = Vec::with_capacity(icons_rendered_here);
    for _ in 0..icons_rendered_here {
        if let Some((icon, tooltip_str)) = icons_iter.by_ref().next() {
            icons.push(icon_tooltip(icon, tooltip_str, tooltip_position, theme).into());
        }
    }

    // if there is just 0 element it will take away the icon padding so it can be negative
    // ensure it is positive
    let space_used = (icons.len() as f32) * PX_PER_ICON + -SPACE_BETWEEN_ICONS.max(0.0);

    let padding = (space_available - space_used) / 2.0;

    (icons, padding)
}

impl<'app> SelectionIcons<'app> {
    /// Render icons around the selection border
    // TODO: Currently, this function does not handle the case where the selection has the
    // same size as the entire screen - so no icons can be rendered at all.
    //
    // We should add even more fallbacks so that it can render a little bit inside of the selection.
    pub fn view(self) -> Element<'app, Message> {
        let icons = vec![
            (
                icon!(Fullscreen),
                crate::Command::Selection(super::selection::Command::SelectRegion {
                    selection: LazyRectangle::FULL,
                }),
                "Select entire monitor (F11)",
            ),
            (
                icon!(Clipboard),
                crate::Command::ImageUpload(crate::image::action::Command::CopyToClipboard),
                "Copy to Clipboard (Enter)",
            ),
            (
                icon!(Save),
                crate::Command::ImageUpload(crate::image::action::Command::SaveScreenshot),
                "Save Screenshot (Ctrl + s)",
            ),
            (
                icon!(Close),
                crate::Command::App(app::Command::Exit),
                "Exit (esc)",
            ),
            if self.app.is_uploading_image {
                // how many seconds we are into the current spin
                let current_spin_secs = self.app.time_elapsed.as_secs_f32() % 2.0;
                // how much % we are through the current spin
                let current_spin_percent = current_spin_secs / 2.0;

                (
                    icon!(Spinner).rotation(Rotation::Floating(Radians(
                        current_spin_percent * f32::consts::TAU,
                    ))),
                    // TODO: Clicking this should cancel the image upload
                    crate::Command::App(app::Command::NoOp),
                    "Screenshot is being uploaded...",
                )
            } else {
                (
                    icon!(Upload),
                    crate::Command::ImageUpload(crate::image::action::Command::UploadScreenshot),
                    "Upload Screenshot (Ctrl + u)",
                )
            },
        ]
        .into_iter()
        .map(|(icon, action, label)| {
            (
                selection_icon(icon, &self.app.config.theme)
                    .on_press(Message::Command {
                        action,
                        // Count does not actually matter at all, since it does not make sense to
                        // do any of the buttons multiple times.
                        count: 1,
                    })
                    .into(),
                label,
            )
        });

        let is_enough_space_at_bottom = self.image_height
            - (self.selection_rect.y + self.selection_rect.height)
            > ICON_BUTTON_SIZE;
        let is_enough_space_at_right = self.image_width
            - (self.selection_rect.x + self.selection_rect.width)
            > ICON_BUTTON_SIZE;
        let is_enough_space_at_top = self.selection_rect.y > ICON_BUTTON_SIZE;
        let is_enough_space_at_left = self.selection_rect.x > ICON_BUTTON_SIZE;

        let icons_len = icons.len();
        let mut icons_iter = icons.into_iter();
        let mut total_icons_positioned = 0;

        // first position the icons on each side (bottom -> right -> top -> left)
        // (bottom_icons, mut bottom_padding)

        let bottom_icons = is_enough_space_at_bottom.then(|| {
            position_icons_in_line(
                self.selection_rect.width,
                tooltip::Position::Bottom,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let right_icons = is_enough_space_at_right.then(|| {
            position_icons_in_line(
                self.selection_rect.height,
                tooltip::Position::Right,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let top_icons = is_enough_space_at_top.then(|| {
            position_icons_in_line(
                self.selection_rect.width,
                tooltip::Position::Top,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let left_icons = is_enough_space_at_left.then(|| {
            position_icons_in_line(
                self.selection_rect.height,
                tooltip::Position::Left,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        // if we reach here, our selection is to small to nicely
        // render all of the icons so we must "stack" them somehow

        // for the 4 sides, combined they will fit at LEAST 8 icons (3 top 3 bottom 1 right 1 left)

        let bottom_icons = bottom_icons.map(|(bottom_icons, bottom_padding)| {
            add_icons_until_there_is_at_least_n_of_them::<MIN_TOP_BOTTOM_ICONS>(
                bottom_icons,
                &mut icons_iter,
                bottom_padding,
                &mut total_icons_positioned,
                tooltip::Position::Bottom,
                &self.app.config.theme,
            )
        });

        let top_icons = top_icons.map(|(top_icons, top_padding)| {
            add_icons_until_there_is_at_least_n_of_them::<MIN_TOP_BOTTOM_ICONS>(
                top_icons,
                &mut icons_iter,
                top_padding,
                &mut total_icons_positioned,
                tooltip::Position::Top,
                &self.app.config.theme,
            )
        });

        let left_icons = left_icons.map(|(left_icons, left_padding)| {
            add_icons_until_there_is_at_least_n_of_them::<MIN_SIDE_ICONS>(
                left_icons,
                &mut icons_iter,
                left_padding,
                &mut total_icons_positioned,
                tooltip::Position::Left,
                &self.app.config.theme,
            )
        });

        let right_icons = right_icons.map(|(right_icons, right_padding)| {
            add_icons_until_there_is_at_least_n_of_them::<MIN_SIDE_ICONS>(
                right_icons,
                &mut icons_iter,
                right_padding,
                &mut total_icons_positioned,
                tooltip::Position::Right,
                &self.app.config.theme,
            )
        });

        // position two additional rows of icons on top and bottom
        // if we STILL have extra icons left

        let extra_top_icons = is_enough_space_at_top.then(|| {
            position_icons_in_line(
                self.selection_rect.width,
                tooltip::Position::Top,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let extra_bottom_icons = is_enough_space_at_bottom.then(|| {
            position_icons_in_line(
                self.selection_rect.width,
                tooltip::Position::Bottom,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let extra_bottom_icons =
            extra_bottom_icons.map(|(extra_bottom_icons, extra_bottom_padding)| {
                add_icons_until_there_is_at_least_n_of_them::<MIN_TOP_BOTTOM_ICONS>(
                    extra_bottom_icons,
                    &mut icons_iter,
                    extra_bottom_padding,
                    &mut total_icons_positioned,
                    tooltip::Position::Bottom,
                    &self.app.config.theme,
                )
            });

        let extra_top_icons = extra_top_icons.map(|(extra_top_icons, extra_top_padding)| {
            add_icons_until_there_is_at_least_n_of_them::<MIN_TOP_BOTTOM_ICONS>(
                extra_top_icons,
                &mut icons_iter,
                extra_top_padding,
                &mut total_icons_positioned,
                tooltip::Position::Top,
                &self.app.config.theme,
            )
        });

        let extra_extra_top_icons = is_enough_space_at_top.then(|| {
            position_icons_in_line(
                self.selection_rect.width,
                tooltip::Position::Top,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let extra_extra_bottom_icons = is_enough_space_at_bottom.then(|| {
            position_icons_in_line(
                self.selection_rect.width,
                tooltip::Position::Bottom,
                &mut total_icons_positioned,
                &mut icons_iter,
                icons_len,
                &self.app.config.theme,
            )
        });

        let extra_extra_top_icons =
            extra_extra_top_icons.map(|(extra_extra_top_icons, extra_extra_top_padding)| {
                add_icons_until_there_is_at_least_n_of_them::<MIN_TOP_BOTTOM_ICONS>(
                    extra_extra_top_icons,
                    &mut icons_iter,
                    extra_extra_top_padding,
                    &mut total_icons_positioned,
                    tooltip::Position::Top,
                    &self.app.config.theme,
                )
            });

        let extra_extra_bottom_icons = extra_extra_bottom_icons.map(
            |(extra_extra_bottom_icons, extra_extra_bottom_padding)| {
                add_icons_until_there_is_at_least_n_of_them::<MIN_TOP_BOTTOM_ICONS>(
                    extra_extra_bottom_icons,
                    &mut icons_iter,
                    extra_extra_bottom_padding,
                    &mut total_icons_positioned,
                    tooltip::Position::Bottom,
                    &self.app.config.theme,
                )
            },
        );

        let right_icons = right_icons.map(|(right_icons, right_padding)| {
            Column::from_vec(right_icons)
                .spacing(SPACE_BETWEEN_ICONS)
                .width(PX_PER_ICON)
                .padding(Padding::default().top(right_padding))
        });

        let left_icons = left_icons.map(|(left_icons, left_padding)| {
            Column::from_vec(left_icons)
                .spacing(SPACE_BETWEEN_ICONS)
                .width(PX_PER_ICON)
                .padding(Padding::default().top(left_padding))
        });

        // there is no way to get amount of children
        // from a Row. that would be prety useful
        let mut top_icon_rows_count = 0;
        let top_icons: Column<_> = extra_extra_top_icons
            .into_iter()
            .chain(extra_top_icons)
            .chain(top_icons)
            .filter_map(|(icons, padding)| {
                (!icons.is_empty()).then(|| {
                    top_icon_rows_count += 1;
                    row![
                        Space::with_width(self.selection_rect.x),
                        Row::from_vec(icons)
                            .spacing(SPACE_BETWEEN_ICONS)
                            .height(PX_PER_ICON)
                            .padding(Padding::default().left(padding))
                    ]
                    .into()
                })
            })
            .collect();

        let bottom_icons: Column<_> = bottom_icons
            .into_iter()
            .chain(extra_bottom_icons)
            .chain(extra_extra_bottom_icons)
            .filter_map(|(icons, padding)| {
                (!icons.is_empty()).then(|| {
                    row![
                        Space::with_width(self.selection_rect.x),
                        Row::from_vec(icons)
                            .spacing(SPACE_BETWEEN_ICONS)
                            .height(PX_PER_ICON)
                            .padding(Padding::default().left(padding))
                    ]
                    .into()
                })
            })
            .collect();

        // include the frame so the icons do not touch the frame
        let selection_height = FRAME_WIDTH * 2.0 + self.selection_rect.height;

        // the left and right rows should be large enough to have at least 1 icon
        // always.
        let height_added = (PX_PER_ICON - selection_height).max(0.0);

        iced::widget::column![
            // just whitespace necessary to align the icons to the selection
            Space::with_height(Length::Fixed(
                (top_icon_rows_count as f32) * -PX_PER_ICON + self.selection_rect.y
                    - height_added / 2.0
            ))
            .width(Fill),
            // top icon row
            top_icons,
            // right icon row + left icon row
            row![Space::with_width(self.selection_rect.x - PX_PER_ICON).height(Fill),]
                .push_maybe(left_icons)
                .push(Space::with_width(FRAME_WIDTH * 2.0 + self.selection_rect.width).height(Fill))
                .push_maybe(right_icons)
                .padding(Padding::default().top(height_added / 2.0))
                .height(selection_height + height_added),
            // bottom icon row
            bottom_icons,
        ]
        .into()
    }
}
