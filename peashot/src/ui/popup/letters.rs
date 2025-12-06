//! Render letters around the screen

use std::iter;

use iced::{
    Element, Event, Font,
    Length::Fill,
    Point, Size, Task,
    font::Weight,
    keyboard::Key,
    widget::{
        Action, Canvas,
        canvas::{self, Path, Stroke},
    },
};

use crate::ui::selection::Selection;

use super::Popup;

crate::declare_commands! {
    enum Command {
        /// Open a grid of letters to pick the top left corner in 3 keystrokes
        PickTopLeftCorner,
        /// Open a grid of letters to pick the bottom right corner in 3 keystrokes
        PickBottomRightCorner
    }
}

impl crate::command::Handler for Command {
    fn handle(self, app: &mut crate::App, _count: u32) -> Task<crate::Message> {
        match self {
            Self::PickTopLeftCorner => {
                app.popup = Some(Popup::Letters(State {
                    picking_corner: PickCorner::TopLeft,
                }));
            }
            Self::PickBottomRightCorner => {
                app.popup = Some(Popup::Letters(State {
                    picking_corner: PickCorner::BottomRight,
                }));
            }
        }

        Task::none()
    }
}

/// State of the letters
#[derive(Debug)]
pub struct State {
    /// Shows a grid of letters on the screen, pressing 3 letters in a row
    /// allows accessing 25 * 25 * 25 = 15,625 different locations
    pub picking_corner: PickCorner,
}

/// Letters message
#[derive(Clone, Debug)]
pub enum Message {
    /// A region was picked using `Letters` widget
    ///
    /// See `LetterLevel` for more info on "level" and "region"
    Pick {
        /// the center of the region clicked on the 3rd level of `Letters`
        point: Point,
        /// The corner which was picked
        corner: PickCorner,
    },
}

impl crate::message::Handler for Message {
    fn handle(self, app: &mut crate::App) -> Task<crate::Message> {
        match self {
            Self::Pick { point, corner } => {
                let sel = app.selection.map_or_else(
                    || {
                        // Intentionally do not increment `app.selections`, because
                        // when selectiong a `0,0` point we do not want to active `--accept-on-select`
                        Selection::new(
                            Point::default(),
                            &app.config.theme,
                            false,
                            app.cli.accept_on_select,
                        )
                    },
                    Selection::norm,
                );
                let x = point.x;
                let y = point.y;
                let new_sel = match corner {
                    PickCorner::TopLeft => {
                        sel.with_x(|_| x)
                            .with_y(|_| y)
                            // make sure that the selection is not going to be out of bounds
                            .with_width(|w| w.min(app.image.width() as f32 - x))
                            .with_height(|h| h.min(app.image.height() as f32 - y))
                    }
                    PickCorner::BottomRight => sel
                        .with_height(|_| y - sel.rect.y)
                        .with_width(|_| x - sel.rect.x),
                };
                app.selection = Some(new_sel);

                if let Some(on_select) = app.cli.accept_on_select {
                    if new_sel.size() != Size::ZERO {
                        if app.selections_created == 0 {
                            return Task::done(crate::Message::Command {
                                action: on_select.into_key_action(),
                                count: 1,
                            });
                        }
                        app.selections_created += 1;
                    }
                }
                app.popup = None;
            }
        }

        Task::none()
    }
}

/// How many letters to draw vertically
const VERTICAL_COUNT: f32 = 5.0;
/// How many letters to draw horizontally
const HORIZONTAL_COUNT: f32 = 5.0;
/// where does `a` start?
const UNICODE_CODEPOINT_LOWERCASE_A_START: u32 = 97;
/// A tiny error margin for doing less than / greater than calculations
const ERROR_MARGIN: f32 = 0.001;

/// How large the font should be
#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum FontSize {
    /// A fixed font size in pixels
    Fixed(f32),
    /// The font size will fill the entire area
    ///
    /// Use this when the font size will be very small so it needs to be easy to see
    Fill,
}

/// Draw letters in a box
#[expect(clippy::too_many_arguments, reason = "todo: refactor")]
fn draw_boxes(
    x_start: f32,
    y_start: f32,
    width: f32,
    height: f32,
    frame: &mut canvas::Frame,
    font_size: FontSize,
    line_width: f32,
    app: &crate::App,
) {
    // We need to offset drawing each line, otherwise it will draw *half* of the line at each side
    let line_offset = line_width / 2.0;

    // `box` = the box which contains a single letter
    let box_width = width / HORIZONTAL_COUNT;
    let box_height = height / VERTICAL_COUNT;

    for x in iter::successors(Some(x_start), |x| {
        (*x + ERROR_MARGIN < x_start + width - box_width).then_some(x + box_width)
    }) {
        for y in iter::successors(Some(y_start), |y| {
            (*y + ERROR_MARGIN < y_start + height - box_height).then_some(y + box_height)
        }) {
            let boxes_drawn = (((x - x_start) / box_width) * HORIZONTAL_COUNT
                + ((y - y_start) / box_height))
                .round() as u32;

            frame.fill_text(canvas::Text {
                content: char::from_u32(UNICODE_CODEPOINT_LOWERCASE_A_START + boxes_drawn)
                    .expect("valid utf8 character")
                    .to_string(),
                position: Point {
                    x: x + box_width / 2.0 - line_offset,
                    y: y + box_height / 2.0 - line_offset,
                },
                font: {
                    let mut font = Font::MONOSPACE;
                    if font_size == FontSize::Fill {
                        font.weight = Weight::Bold;
                    }
                    font
                },
                color: app.config.theme.letters_fg,
                size: match font_size {
                    FontSize::Fixed(px) => px,
                    FontSize::Fill => box_height,
                }
                .into(),
                align_x: iced::alignment::Horizontal::Center,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }

        // draw vertical lines
        frame.stroke(
            &Path::line(
                Point::new(x + line_offset, y_start),
                Point::new(x + line_offset, y_start + height),
            ),
            Stroke {
                style: app.config.theme.letters_lines.into(),
                width: line_width,
                ..Default::default()
            },
        );
    }

    // draw horizontal lines
    for y in iter::successors(Some(y_start), |y| {
        (*y + ERROR_MARGIN < y_start + height).then_some(y + box_height)
    }) {
        frame.stroke(
            &Path::line(
                Point::new(x_start, y + line_offset),
                Point::new(x_start + width, y + line_offset),
            ),
            Stroke {
                style: app.config.theme.letters_lines.into(),
                width: line_width,
                ..Default::default()
            },
        );
    }

    // draw 2 extra lines at the end of each axis, so we have
    // lines on each side of equal thickness and its nice and symmetrical

    // horizontal line at the end
    frame.stroke(
        &Path::line(
            Point::new(x_start + width - line_offset, y_start),
            Point::new(x_start + width - line_offset, y_start + height),
        ),
        Stroke {
            style: app.config.theme.letters_lines.into(),
            width: line_width,
            ..Default::default()
        },
    );
    // vertical line at the end
    frame.stroke(
        &Path::line(
            Point::new(x_start, y_start + height - line_offset),
            Point::new(x_start + width, y_start + height - line_offset),
        ),
        Stroke {
            style: app.config.theme.letters_lines.into(),
            width: line_width,
            ..Default::default()
        },
    );
}

/// Level of the letter grid.
///
/// The letter grid consists of 3 "levels"
///
/// - Level 1: the entire screen is divided into 25 regions, a letter is assigned to each
///   region. When we input a letter, 1 of the 25 regions is picked and we progress onto level 2.
/// - Level 2: The region that we picked is further divided into 25 smaller regions. A single letter
///   is assigned to each region once again. Inputting another letter progresses us to Level 3.
/// - Level 3: The region picked in Level 2 is further divided into 25 even tinier regions. Now, once we
///   pick any of the tiny regions the center of that region will be sent as a `Message` to the main
///   `App`.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum LetterLevel {
    /// First level
    #[default]
    First,
    /// Second click on the letter grid
    /// Choose a more precise location than the first
    Second {
        /// top left corner of the region clicked during `First`
        point: Point,
    },
    /// Third click on the letter grid
    /// Once we click this, it's finished and we will notify the `App`
    Third {
        /// top left corner of the region clicked during `Second`
        point: Point,
    },
}

/// When a position is picked, what does that signify?
///
/// This enum represents the possible outcomes that can happen when we pick a position.
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug)]
pub enum PickCorner {
    /// Picking position for the top-left corner of the selection
    TopLeft,
    /// Picking position for the bottom-right corner of the selection
    BottomRight,
}

/// Letters
#[derive(Clone, Copy, Debug)]
pub struct Letters<'app> {
    /// The App
    pub app: &'app crate::App,
    /// Corner to pick the position for
    pub pick_corner: PickCorner,
}

impl<'app> Letters<'app> {
    /// Render a grid of letters
    pub fn view(self) -> Element<'app, crate::Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }
}

impl canvas::Program<crate::Message> for Letters<'_> {
    type State = LetterLevel;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill_rectangle(
            bounds.position(),
            bounds.size(),
            self.app.config.theme.letters_bg,
        );

        let x_start = 0.0;
        let y_start = 0.0;
        let width = frame.width();
        let height = frame.height();

        match state {
            LetterLevel::First => draw_boxes(
                x_start,
                y_start,
                width,
                height,
                &mut frame,
                FontSize::Fixed(48.0),
                1.0,
                self.app,
            ),
            LetterLevel::Second { point } => draw_boxes(
                point.x,
                point.y,
                width / HORIZONTAL_COUNT,
                height / VERTICAL_COUNT,
                &mut frame,
                FontSize::Fixed(32.0),
                1.0,
                self.app,
            ),
            LetterLevel::Third { point } => draw_boxes(
                point.x,
                point.y,
                width / HORIZONTAL_COUNT.powi(2),
                height / VERTICAL_COUNT.powi(2),
                &mut frame,
                FontSize::Fill,
                0.2,
                self.app,
            ),
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Option<Action<crate::Message>> {
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            modified_key: Key::Character(input),
            ..
        }) = event
        {
            if let Some(ch) = input.chars().next() {
                let ch = ch as u32 - UNICODE_CODEPOINT_LOWERCASE_A_START;
                let vertical_steps = (ch % VERTICAL_COUNT as u32) as f32;
                let horizontal_steps = (ch / HORIZONTAL_COUNT as u32) as f32;
                match state {
                    LetterLevel::First => {
                        let box_width = bounds.width / HORIZONTAL_COUNT;
                        let box_height = bounds.height / VERTICAL_COUNT;

                        *state = LetterLevel::Second {
                            point: Point {
                                x: horizontal_steps * box_width,
                                y: vertical_steps * box_height,
                            },
                        };

                        return Some(Action::request_redraw());
                    }
                    LetterLevel::Second { point } => {
                        let box_width = bounds.width / HORIZONTAL_COUNT.powi(2);
                        let box_height = bounds.height / VERTICAL_COUNT.powi(2);

                        *state = LetterLevel::Third {
                            point: Point {
                                x: horizontal_steps * box_width + point.x,
                                y: vertical_steps * box_height + point.y,
                            },
                        };

                        return Some(Action::request_redraw());
                    }
                    LetterLevel::Third { point } => {
                        let box_width = bounds.width / HORIZONTAL_COUNT.powi(3);
                        let box_height = bounds.height / VERTICAL_COUNT.powi(3);

                        return Some(Action::publish(crate::Message::Letters(Message::Pick {
                            // INFO: We want the point to be in the center, unlike in the previous levels where
                            // we wanted the top-left corner
                            point: Point {
                                x: horizontal_steps * box_width + point.x + box_width / 2.0,
                                y: vertical_steps * box_height + point.y + box_height / 2.0,
                            },
                            corner: self.pick_corner,
                        })));
                    }
                }
            }
        } else if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(iced::keyboard::key::Named::Escape),
            ..
        }) = event
        {
            return Some(Action::publish(crate::Message::ClosePopup));
        }

        // Any unrecognized event should not propagate to the `App`
        Some(Action::capture())
    }
}
