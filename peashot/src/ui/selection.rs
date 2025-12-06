//! A `Selection` is the structure representing a selected area in the background image
use crate::geometry::Corners;
use crate::geometry::Direction;
use crate::geometry::RectangleExt as _;
use crate::geometry::Side;
use crate::geometry::SideOrCorner;
use crate::lazy_rect::LazyRectangle;
use delegate::delegate;
use iced::Task;
use iced::mouse::Cursor;
use iced::mouse::Interaction;
use iced::widget::Action;
use iced::widget::canvas;
use iced::{Point, Rectangle, Size};

/// A place on the rectangle
#[derive(ferrishot_knus::DecodeScalar, Debug, Clone, PartialEq, Copy, Eq, Ord, PartialOrd)]
pub enum Place {
    /// Center
    Center,
    /// Center on the x-axis
    XCenter,
    /// Center on the y-axis
    YCenter,
    /// Top-left corner
    TopLeft,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
    /// Top-right corner
    TopRight,
    /// Left side
    Left,
    /// Right side
    Right,
    /// Top side
    Top,
    /// Bottom side
    Bottom,
}

crate::declare_commands! {
    enum Command {
        /// Set the width to whatever number is currently pressed
        SetWidth,
        /// Set the height to whatever number is currently pressed
        SetHeight,
        /// Set selection to encompass the entire screen
        SelectRegion {
            #[ferrishot_knus(str)]
            selection: LazyRectangle,
        },
        /// Remove the selection
        ClearSelection,
        /// Shift the selection in the given direction by pixels
        Move {
            direction: Direction,
            amount: u32 = u32::MAX,
        },
        /// Increase the size of the selection in the given direction by pixels
        Extend {
            direction: Direction,
            amount: u32 = u32::MAX,
        },
        /// Decrease the size of the selection in the given direction by pixels
        Shrink {
            direction: Direction,
            amount: u32 = u32::MAX,
        },
        /// Move rectangle to a place
        Goto {
            place: Place,
        }
    }
}

impl crate::command::Handler for Command {
    fn handle(self, app: &mut crate::App, count: u32) -> Task<crate::Message> {
        match self {
            Self::SetWidth => {
                let Some(selection) = app.selection.as_mut() else {
                    app.errors.push("Nothing is selected.");
                    return Task::none();
                };
                let image_width = app.image.width() as f32;
                let sel = selection.norm();

                *selection = sel.with_width(|_| (count as f32).min(image_width - sel.rect.x));
            }
            Self::SetHeight => {
                let Some(selection) = app.selection.as_mut() else {
                    app.errors.push("Nothing is selected.");
                    return Task::none();
                };
                let image_height = app.image.height() as f32;
                let sel = selection.norm();

                *selection = sel.with_height(|_| (count as f32).min(image_height - sel.rect.y));
            }
            Self::SelectRegion { selection } => {
                let rect = selection.init(app.image.bounds());
                app.selection = Some(
                    Selection::new(
                        rect.top_left(),
                        &app.config.theme,
                        app.selections_created == 0,
                        app.cli.accept_on_select,
                    )
                    .with_size(|_| rect.size()),
                );
                app.selections_created += 1;
            }
            Self::ClearSelection => {
                app.selection = None;
            }
            Self::Move { direction, amount } => {
                let Some(selection) = app.selection.as_mut() else {
                    app.errors.push("Nothing is selected.");
                    return Task::none();
                };
                let image_width = app.image.width() as f32;
                let image_height = app.image.height() as f32;
                let amount = amount as f32 * count as f32;
                let sel = selection.norm();

                *selection = match direction {
                    Direction::Up => sel.with_y(|y| (y - amount).max(0.0)),
                    Direction::Down => {
                        sel.with_y(|y| (y + amount).min(image_height - sel.rect.height))
                    }
                    Direction::Left => sel.with_x(|x| (x - amount).max(0.0)),
                    Direction::Right => {
                        sel.with_x(|x| (x + amount).min(image_width - sel.rect.width))
                    }
                }
            }
            Self::Extend { direction, amount } => {
                let Some(selection) = app.selection.as_mut() else {
                    app.errors.push("Nothing is selected.");
                    return Task::none();
                };
                let image_height = app.image.height() as f32;
                let image_width = app.image.width() as f32;
                let sel = selection.norm();
                let amount = amount as f32 * count as f32;

                *selection = match direction {
                    Direction::Up => sel
                        .with_y(|y| (y - amount).max(0.0))
                        .with_height(|h| (h + amount).min(sel.rect.y + sel.rect.height)),
                    Direction::Down => {
                        sel.with_height(|h| (h + amount).min(image_height - sel.rect.y))
                    }
                    Direction::Left => sel
                        .with_x(|x| (x - amount).max(0.0))
                        .with_width(|w| (w + amount).min(sel.rect.x + sel.rect.width)),
                    Direction::Right => {
                        sel.with_width(|w| (w + amount).min(image_width - sel.rect.x))
                    }
                }
            }
            Self::Shrink { direction, amount } => {
                let Some(selection) = app.selection.as_mut() else {
                    app.errors.push("Nothing is selected.");
                    return Task::none();
                };
                let sel = selection.norm();
                let amount = amount as f32 * count as f32;

                *selection = match direction {
                    Direction::Up => sel
                        .with_y(|y| (y + amount).min(sel.rect.y + sel.rect.height))
                        .with_height(|h| (h - amount).max(0.0)),
                    Direction::Down => sel.with_height(|h| (h - amount).max(0.0)),
                    Direction::Left => sel
                        .with_x(|x| (x + amount).min(sel.rect.x + sel.rect.width))
                        .with_width(|w| (w - amount).max(0.0)),
                    Direction::Right => sel.with_width(|w| (w - amount).max(0.0)),
                }
            }
            Self::Goto { place } => {
                let Some(selection) = app.selection.as_mut() else {
                    app.errors.push("Nothing is selected.");
                    return Task::none();
                };
                let image_height = app.image.height() as f32;
                let image_width = app.image.width() as f32;

                match place {
                    Place::Center => {
                        selection.rect.x = (image_width - selection.rect.width) / 2.0;
                        selection.rect.y = (image_height - selection.rect.height) / 2.0;
                    }
                    Place::XCenter => {
                        selection.rect.x = (image_width - selection.rect.width) / 2.0;
                    }
                    Place::YCenter => {
                        selection.rect.y = (image_height - selection.rect.height) / 2.0;
                    }
                    Place::TopLeft => {
                        selection.rect.x = 0.0;
                        selection.rect.y = 0.0;
                    }
                    Place::TopRight => {
                        selection.rect.x = image_width - selection.rect.width;
                        selection.rect.y = 0.0;
                    }
                    Place::BottomLeft => {
                        selection.rect.x = 0.0;
                        selection.rect.y = image_height - selection.rect.height;
                    }
                    Place::BottomRight => {
                        selection.rect.x = image_width - selection.rect.width;
                        selection.rect.y = image_height - selection.rect.height;
                    }
                    Place::Top => {
                        selection.rect.y = 0.0;
                    }
                    Place::Bottom => {
                        selection.rect.y = image_height - selection.rect.height;
                    }
                    Place::Left => {
                        selection.rect.x = 0.0;
                    }
                    Place::Right => {
                        selection.rect.x = image_width - selection.rect.width;
                    }
                }
            }
        }

        Task::none()
    }
}

/// Message for a selection
#[derive(Clone, Debug)]
pub enum Message {
    /// The selection is currently being resized
    Resize {
        /// Current position of the cursor
        current_cursor_pos: Point,
        /// Initial position of the cursor
        initial_cursor_pos: Point,
        /// Which side we are currently resizing
        resize_side: SideOrCorner,
        /// Selection rectangle as it looked like when we just started resizing
        initial_rect: Rectangle,
        /// A key to obtain `&mut Selection` from `Option<Selection>` with a guarantee that it will
        /// always be there (to bypass the limitation that we cannot pass `&mut Selection` in a `Message`)
        sel_is_some: SelectionIsSome,
        /// Multiplier for how fast we are resizing.
        speed: Speed,
    },
    /// Update status of existing selection
    UpdateStatus(SelectionStatus, SelectionIsSome),
    /// Create a zero size selection
    CreateSelection(Point),
    /// Left mouse is held down and dragged
    ///
    /// Contains the new point of the mouse
    MoveSelection {
        /// Current position of the cursor
        current_cursor_pos: Point,
        /// Position of the cursor when we first started moving the selection
        initial_cursor_pos: Point,
        /// Current selection
        current_selection: Selection,
        /// Top-left corner of the selection before we started moving it
        initial_rect_pos: Point,
        /// How fast the selection should move
        speed: Speed,
    },
    /// Enter idle mode
    EnterIdle,
    /// When we have not yet released the left mouse button
    /// and are dragging the selection to extend it
    ExtendNewSelection(Point),
    /// Holding right-click, the selection will move the
    /// nearest corner to the cursor
    ResizeToCursor {
        /// Current position of the cursor
        cursor_pos: Point,
        /// Current selection
        selection: Selection,
        /// A key to obtain `&mut Selection` from `Option<Selection>` with a guarantee that it will
        /// always be there (to bypass the limitation that we cannot pass `&mut Selection` in a `Message`)
        sel_is_some: SelectionIsSome,
    },
}

impl crate::message::Handler for Message {
    fn handle(self, app: &mut crate::App) -> Task<crate::Message> {
        match self {
            Self::CreateSelection(point) => {
                app.selection = Some(
                    Selection::new(
                        point,
                        &app.config.theme,
                        app.selections_created == 0,
                        app.cli.accept_on_select,
                    )
                    .with_status(SelectionStatus::Create),
                );
                app.selections_created += 1;
            }
            Self::UpdateStatus(status, sel_is_some) => {
                let sel = app.selection.unlock(sel_is_some);
                sel.status = status;
            }
            Self::EnterIdle => {
                if let Some(selection) = app.selection.as_mut() {
                    selection.status = SelectionStatus::Idle;
                }
            }
            Self::ExtendNewSelection(new_mouse_position) => {
                app.selection = app.selection.take().map(|selected_region| {
                    let width = new_mouse_position.x - selected_region.rect.x;
                    let height = new_mouse_position.y - selected_region.rect.y;

                    selected_region.with_size(|_| Size { width, height })
                });
            }
            Self::ResizeToCursor {
                cursor_pos,
                selection,
                sel_is_some,
            } => {
                let (corner_point, corners) = selection.corners().nearest_corner(cursor_pos);
                let sel = app.selection.unlock(sel_is_some);

                sel.rect = corners.resize_rect(
                    selection.rect,
                    cursor_pos.y - corner_point.y,
                    cursor_pos.x - corner_point.x,
                );

                sel.status = SelectionStatus::Resize {
                    initial_rect: sel.rect,
                    initial_cursor_pos: cursor_pos,
                    resize_side: SideOrCorner::Corner(corners),
                };
            }
            Self::Resize {
                current_cursor_pos,
                initial_cursor_pos,
                resize_side,
                initial_rect,
                sel_is_some,
                speed,
            } => {
                let selected_region = app.selection.unlock(sel_is_some);
                let resize_speed = speed.speed();

                let dy = (current_cursor_pos.y - initial_cursor_pos.y) * resize_speed;
                let dx = (current_cursor_pos.x - initial_cursor_pos.x) * resize_speed;

                selected_region.rect = match resize_side {
                    SideOrCorner::Side(side) => match side {
                        Side::Top => initial_rect.with_height(|h| h - dy).with_y(|y| y + dy),
                        Side::Right => initial_rect.with_width(|w| w + dx),
                        Side::Bottom => initial_rect.with_height(|h| h + dy),
                        Side::Left => initial_rect.with_width(|w| w - dx).with_x(|x| x + dx),
                    },
                    SideOrCorner::Corner(corner) => corner.resize_rect(initial_rect, dy, dx),
                };

                if speed
                    == (Speed::Slow {
                        has_speed_changed: true,
                    })
                {
                    selected_region.status = SelectionStatus::Resize {
                        initial_rect: selected_region.rect,
                        initial_cursor_pos: current_cursor_pos,
                        resize_side,
                    }
                }
            }
            Self::MoveSelection {
                current_cursor_pos,
                initial_cursor_pos,
                current_selection,
                initial_rect_pos,
                speed,
            } => {
                let mut new_selection = current_selection.with_pos(|_| {
                    initial_rect_pos + ((current_cursor_pos - initial_cursor_pos) * speed.speed())
                });

                let old_x = new_selection.rect.x as u32;
                let old_y = new_selection.rect.y as u32;

                // if any of these actually get changed we are going to set the new selection status.

                new_selection.rect.x = new_selection
                    .rect
                    .x
                    .min(app.image.width() as f32 - new_selection.rect.width)
                    .max(0.0);

                new_selection.rect.y = new_selection
                    .rect
                    .y
                    .min(app.image.height() as f32 - new_selection.rect.height)
                    .max(0.0);

                if new_selection.rect.y as u32 != old_y || new_selection.rect.x as u32 != old_x {
                    new_selection.status = SelectionStatus::Move {
                        initial_rect_pos: new_selection.pos(),
                        initial_cursor_pos: current_cursor_pos,
                    }
                }

                if speed
                    == (Speed::Slow {
                        has_speed_changed: true,
                    })
                {
                    new_selection.status = SelectionStatus::Move {
                        initial_rect_pos: current_selection.pos(),
                        initial_cursor_pos: current_cursor_pos,
                    }
                }

                app.selection = Some(new_selection);
            }
        }

        Task::none()
    }
}

/// The size of the lines of the frame of the selection
pub const FRAME_WIDTH: f32 = 2.0;

/// Size of the button for the icon, which includes the
/// icon itself and space around it (bigger than `ICON_SIZE`)
pub const ICON_BUTTON_SIZE: f32 = 37.0;

/// How fast the selection resizes
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Speed {
    /// Resize follows the cursor. Cursor moves 1px -> the selection resizes by 1px
    Regular,
    /// Resize is slower than the cursor. Cursor moves 1px -> the selection resizes by less than that
    Slow {
        /// The speed was previously different, so the selection status must be updated to sync
        has_speed_changed: bool,
    },
}

impl Speed {
    /// For a given px of cursor movement, how many px does the selection resize by?
    pub const fn speed(self) -> f32 {
        match self {
            Self::Regular => 1.0,
            Self::Slow { .. } => 0.1,
        }
    }
}

/// The selected area of the desktop which will be captured
#[derive(Debug, Copy, Clone)]
pub struct Selection {
    /// If this selection is the first one
    pub is_first: bool,
    /// Accept on select
    pub accept_on_select: Option<crate::image::action::Command>,
    /// Theme of the app
    pub theme: crate::Theme,
    /// Area represented by the selection
    pub rect: Rectangle,
    /// Status of the selection
    pub status: SelectionStatus,
}

/// What the selection is doing at the moment
#[derive(Debug, Default, Clone, Copy, PartialEq, derive_more::IsVariant)]
pub enum SelectionStatus {
    /// The selection is currently being resized
    Resize {
        /// Position of the selection rectangle before we started resizing it
        initial_rect: Rectangle,
        /// Cursor position before we started resizing it
        initial_cursor_pos: Point,
        /// The side or corner being resized
        resize_side: SideOrCorner,
    },
    /// The selection is currently being moved entirely
    ///
    /// left click + hold + move mouse
    Move {
        /// Top-left point of the selection Rect before we started dragging the
        /// selection
        initial_rect_pos: Point,
        /// Position of the cursor when we just started dragging the selection
        initial_cursor_pos: Point,
    },
    /// The selection is currently being created, e.g.
    /// hold left click and drag
    Create,
    /// The selection is not moving
    #[default]
    Idle,
}

/// The existance of this struct guarantees that an `Option<Selection>` is always `Some`.
///
/// We have this because very often in the app we want to pass the knowledge that our `Selection`
/// exists through a `crate::Message`, however it is not possible to do that
///
/// For example, we send `crate::Message::Foo` from `<App as canvas::Program<crate::Message>>::update` if, and only if `App.selection.is_some()`.
///
/// Inside of `App::update` we receive this message and we have access to a `&mut App`. We need to
/// modify the selection and we are certain that it exists. Yet we must still use an `unwrap`.
///
/// This module prevents that. When obtaining a `Selection` from an `App`, we also get a `SelectionIsSome`.
/// This struct is only possible to construct from the `Option<Selection>::get` method.
///
/// This adds a little bit of complexity in exchange for preventing dozens of `expect`/`unwrap`s in the app and a type-safe way of guaranteeing that `Selection` exists.
///
/// # Important
///
/// This struct should *never* be created manually. It should only ever be obtained from the
/// `Option<&mut Selection>::get` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionIsSome {
    /// Private field makes this type impossible to construct outside of the module it is defined in
    _private: (),
}

/// Methods for extracting value from an optional selection,
/// with a guarantee that it can never be None.
#[easy_ext::ext(OptionalSelectionExt)]
pub impl Option<Selection> {
    /// Attempt to get the inner selection. if successful, return a key that allows opening
    /// this option again with a guarantee for existance.
    fn get(self) -> Option<(Selection, SelectionIsSome)> {
        self.map(|x| (x, SelectionIsSome { _private: () }))
    }
    /// Extract the selection, with a guarantee that it is always there
    fn unlock(&mut self, _key: SelectionIsSome) -> &mut Selection {
        self.as_mut()
            .expect("Cannot be None if the key is provided")
    }
}

impl Selection {
    /// Create the initial selection
    pub fn initial(
        rect: Rectangle,
        theme: &crate::Theme,
        accept_on_select: Option<crate::image::action::Command>,
    ) -> Self {
        Self {
            is_first: true,
            accept_on_select,
            theme: *theme,
            rect,
            status: SelectionStatus::Idle,
        }
    }

    /// Set a theme to the selection
    pub fn with_theme(mut self, theme: &crate::Theme) -> Self {
        self.theme = *theme;
        self
    }

    /// Draw the `Selection`
    pub fn draw(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        self.draw_shade(frame, bounds);
        self.draw_border(frame);
        self.draw_corners(frame);
    }

    /// Type of the mouse cursor
    pub fn mouse_interaction(&self, cursor: Cursor) -> Interaction {
        // if we are already resizing, then this cursor takes priority
        // e.g. we are resizing horizontally but we are on the top left
        // corner = we should have horizontal resize cursor.
        (if let SelectionStatus::Resize { resize_side, .. } = self.status {
            // resize icon corresponding to a specific side
            Some(resize_side.mouse_icon())
        } else if self.status.is_move() {
            Some(Interaction::Grabbing)
        } else {
            None
        })
        .or_else(|| {
            // when we started dragging a side, even if we go outside of the bounds of that side (which
            // happens often when we are dragging the mouse fast), we don't want the cursor to change
            cursor
                .position()
                .and_then(|cursor| self.corners().side_at(cursor).map(SideOrCorner::mouse_icon))
        })
        .unwrap_or_else(|| {
            if self.cursor_in_selection(cursor).is_some() {
                Interaction::Grab
            } else {
                Interaction::Crosshair
            }
        })
    }

    /// Draw shade around the selection
    pub fn draw_shade(&self, frame: &mut canvas::Frame, image_bounds: Rectangle) {
        let sel = self.norm();

        // represents the area outside of the selection
        let outside = canvas::Path::new(|p| {
            p.move_to(image_bounds.top_left());
            p.line_to(image_bounds.top_right());
            p.line_to(image_bounds.bottom_right());
            p.line_to(image_bounds.bottom_left());
            p.move_to(image_bounds.top_left());

            p.move_to(sel.top_left());
            p.line_to(sel.bottom_left());
            p.line_to(sel.bottom_right());
            p.line_to(sel.top_right());
            p.move_to(sel.top_left());
        });

        frame.fill(&outside, self.theme.non_selected_region);
    }

    /// Renders border of the selection
    pub fn draw_border(&self, frame: &mut canvas::Frame) {
        // Draw the shadow of the border of the selection
        frame.stroke_rectangle(
            self.pos(),
            self.size(),
            canvas::Stroke::default()
                .with_color(self.theme.drop_shadow)
                .with_width(FRAME_WIDTH * 2.0),
        );
        // Draw the border around the selection (the sides)
        frame.stroke_rectangle(
            self.pos(),
            self.size(),
            canvas::Stroke::default()
                .with_color(self.theme.selection_frame)
                .with_width(FRAME_WIDTH),
        );
    }

    /// Render the circles for each side
    pub fn draw_corners(&self, frame: &mut canvas::Frame) {
        /// Radius of each of the 4 corner circles in the frame drawn around the selection
        const FRAME_CIRCLE_RADIUS: f32 = 6.0;

        let corners = self.corners();
        for circle in [
            corners.top_left,
            corners.top_right,
            corners.bottom_left,
            corners.bottom_right,
        ]
        .map(|corner| canvas::Path::circle(corner, FRAME_CIRCLE_RADIUS))
        {
            frame.fill(&circle, self.theme.selection_frame);
        }
    }

    /// Set status of the selection
    pub const fn with_status(mut self, status: SelectionStatus) -> Self {
        self.status = status;
        self
    }

    /// Create selection at a point with a size of zero
    pub fn new(
        point: Point,
        theme: &crate::Theme,
        is_first: bool,
        accept_on_select: Option<crate::image::action::Command>,
    ) -> Self {
        Self {
            rect: Rectangle::new(point, Size::default()),
            status: SelectionStatus::default(),
            theme: *theme,
            is_first,
            accept_on_select,
        }
    }

    /// Update the selection
    pub fn update(
        &self,
        state: &mut SelectionKeysState,
        event: &iced::Event,
        _bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<crate::Message>> {
        use iced::Event::{Keyboard, Mouse, Touch};
        use iced::keyboard::Event::KeyPressed;
        use iced::keyboard::Event::KeyReleased;
        use iced::keyboard::Key::Named;
        use iced::keyboard::key::Named::{Control, Shift};
        use iced::mouse::Button::{Left, Right};
        use iced::mouse::Event::ButtonPressed;
        use iced::mouse::Event::ButtonReleased;
        use iced::mouse::Event::CursorMoved;
        use iced::touch::Event::{FingerLifted, FingerMoved, FingerPressed};

        let message = match event {
            Touch(FingerPressed { .. }) | Mouse(ButtonPressed(Left)) => {
                state.is_left_down = true;

                if let Some((cursor, side)) = cursor.position().and_then(|cursor_pos| {
                    self.corners()
                        .side_at(cursor_pos)
                        .map(|side| (cursor_pos, side))
                }) {
                    // Left click on corners = Start resizing selection
                    crate::Message::Selection(Box::new(Message::UpdateStatus(
                        SelectionStatus::Resize {
                            initial_rect: self.rect.norm(),
                            initial_cursor_pos: cursor,
                            resize_side: side,
                        },
                        SelectionIsSome { _private: () },
                    )))
                } else if let Some((cursor, selected_region)) = self.cursor_in_selection(cursor) {
                    // Left click on selection = Move selection
                    crate::Message::Selection(Box::new(Message::UpdateStatus(
                        SelectionStatus::Move {
                            initial_rect_pos: selected_region.norm().pos(),
                            initial_cursor_pos: cursor,
                        },
                        SelectionIsSome { _private: () },
                    )))
                } else if let Some(cursor_position) = cursor.position() {
                    // Left click outside of selection = Create new selection
                    crate::Message::Selection(Box::new(Message::CreateSelection(cursor_position)))
                } else {
                    return None;
                }
            }
            Touch(FingerLifted { .. }) | Mouse(ButtonReleased(Left)) => {
                state.is_left_down = false;

                self.accept_on_select.map_or_else(
                    // stop the creating of the initial selection
                    || crate::Message::Selection(Box::new(Message::EnterIdle)),
                    |on_select| {
                        if self.is_first && !state.is_ctrl_down {
                            // we have created 1 selections in total, (the current one)
                            crate::Message::Command {
                                action: on_select.into_key_action(),
                                count: 1,
                            }
                        } else {
                            // stop the creating of the initial selection
                            crate::Message::Selection(Box::new(Message::EnterIdle))
                        }
                    },
                )
            }
            Keyboard(KeyPressed {
                key: Named(Control),
                ..
            }) => {
                state.is_ctrl_down = true;
                return None;
            }
            Keyboard(KeyReleased {
                key: Named(Control),
                ..
            }) => {
                state.is_ctrl_down = false;
                return None;
            }
            Keyboard(KeyReleased {
                key: Named(Shift), ..
            }) => {
                state.is_shift_down = false;
                return None;
            }
            Mouse(ButtonPressed(Right)) => {
                state.is_right_down = true;

                crate::Message::Selection(Box::new(Message::ResizeToCursor {
                    cursor_pos: cursor.position()?,
                    selection: self.norm(),
                    sel_is_some: SelectionIsSome { _private: () },
                }))
            }
            Mouse(ButtonReleased(Right)) => {
                state.is_right_down = false;

                crate::Message::Selection(Box::new(Message::EnterIdle))
            }
            Keyboard(KeyPressed {
                key: Named(Shift), ..
            }) => {
                state.is_shift_down = true;

                let current_cursor_pos = cursor.position()?;

                // If we are already resizing a side, and we press shift, we
                // want to act as if we just started resizing from this point again
                // so we do not get a surprising jump
                match self.status {
                    SelectionStatus::Resize { resize_side, .. } => {
                        crate::Message::Selection(Box::new(Message::Resize {
                            resize_side,
                            // start resizing from this point on
                            current_cursor_pos,
                            initial_cursor_pos: current_cursor_pos,
                            // the current selection becomes the new starting point
                            initial_rect: self.rect,
                            sel_is_some: SelectionIsSome { _private: () },
                            speed: Speed::Slow {
                                has_speed_changed: true,
                            },
                        }))
                    }
                    SelectionStatus::Move { .. } => {
                        crate::Message::Selection(Box::new(Message::MoveSelection {
                            current_cursor_pos,
                            initial_cursor_pos: current_cursor_pos,
                            current_selection: *self,
                            initial_rect_pos: self.pos(),
                            speed: Speed::Slow {
                                has_speed_changed: true,
                            },
                        }))
                    }
                    _ => return None,
                }
            }
            Touch(FingerMoved { position, .. }) | Mouse(CursorMoved { position })
                if self.is_resize() =>
            {
                // FIXME: this will not be necessary when we have `let_chains`
                let SelectionStatus::Resize {
                    resize_side,
                    initial_rect,
                    initial_cursor_pos,
                } = self.status
                else {
                    unreachable!("has `.is_some_and(is_resized)` guard");
                };

                crate::Message::Selection(Box::new(Message::Resize {
                    current_cursor_pos: *position,
                    resize_side,
                    initial_cursor_pos,
                    initial_rect,
                    sel_is_some: SelectionIsSome { _private: () },
                    speed: if state.is_shift_down {
                        Speed::Slow {
                            has_speed_changed: false,
                        }
                    } else {
                        Speed::Regular
                    },
                }))
            }
            Touch(FingerMoved { position, .. }) | Mouse(CursorMoved { position })
                if self.is_move() =>
            {
                let current_selection = self.norm();

                // FIXME: this will not be necessary when we have `if_let_guard`
                let SelectionStatus::Move {
                    initial_rect_pos,
                    initial_cursor_pos,
                } = current_selection.status
                else {
                    unreachable!();
                };

                crate::Message::Selection(Box::new(Message::MoveSelection {
                    current_cursor_pos: *position,
                    initial_cursor_pos,
                    current_selection,
                    initial_rect_pos,
                    speed: if state.is_shift_down {
                        Speed::Slow {
                            has_speed_changed: false,
                        }
                    } else {
                        Speed::Regular
                    },
                }))
            }
            Touch(FingerMoved { position, .. }) | Mouse(CursorMoved { position })
                if self.is_create() =>
            {
                crate::Message::Selection(Box::new(Message::ExtendNewSelection(*position)))
            }
            _ => return None,
        };

        Some(Action::publish(message))
    }

    /// If the given cursor intersects the selected region, give the region and
    /// the cursor
    pub fn cursor_in_selection(self, cursor: Cursor) -> Option<(Point, Self)> {
        cursor.position().and_then(|cursor_pos| {
            self.norm()
                .contains(cursor_pos)
                .then_some((cursor_pos, self))
        })
    }

    /// If the given cursor intersects the selected region, give the region and
    /// the cursor
    pub fn cursor_in_selection_mut(&mut self, cursor: Cursor) -> Option<(Point, &mut Self)> {
        cursor.position().and_then(|cursor_pos| {
            self.norm()
                .contains(cursor_pos)
                .then_some((cursor_pos, self))
        })
    }

    delegate! {
        to self.rect {
            /// The height and width of the selection
            pub fn size(self) -> Size;
            /// Top left corner of the selection
            pub fn pos(self) -> Point;
            /// Top-left, top-right, bottom-left and bottom-right points
            pub fn corners(self) -> Corners;
            /// Whether this selection contains a given point
            pub fn contains(self, point: Point) -> bool;

            /// Position of the top left corner
            pub fn top_left(self) -> Point;
            /// Position of the top right corner
            pub fn top_right(self) -> Point;
            /// Position of the bottom right corner
            pub fn bottom_right(self) -> Point;
            /// Position of the bottom left corner
            pub fn bottom_left(self) -> Point;

            /// Center
            pub fn center(self) -> Point;
            /// Top center
            pub fn top_center(self) -> Point;
            /// Right center
            pub fn right_center(self) -> Point;
            /// Left center
            pub fn left_center(self) -> Point;
            /// Bottom center
            pub fn bottom_center(self) -> Point;
        }
        #[expr(self.rect = $; self)]
        to self.rect {
            /// Update the size of the rect
            pub fn with_size<F: FnOnce(Size) -> Size>(mut self, f: F) -> Self;
            /// Update the position of the top left corner
            pub fn with_pos<F: FnOnce(Point) -> Point>(mut self, f: F) -> Self;
            /// Update the selection's height
            pub fn with_height<F: FnOnce(f32) -> f32>(mut self, f: F) -> Self;
            /// Update the selection's width
            pub fn with_width<F: FnOnce(f32) -> f32>(mut self, f: F) -> Self;
            /// Update the x coordinate of the top left corner
            pub fn with_x<F: FnOnce(f32) -> f32>(mut self, f: F) -> Self;
            /// Update the y coordinate of the top left corner
            pub fn with_y<F: FnOnce(f32) -> f32>(mut self, f: F) -> Self;
            /// Make sure the width and height is not negative
            pub fn norm(mut self) -> Self;
        }
        to self.status {
            /// The selection is currently being dragged
            pub const fn is_move(self) -> bool;
            /// The selection is not moving
            pub const fn is_idle(self) -> bool;
            /// The selection is being resized
            pub const fn is_resize(self) -> bool;
            /// The selection is being created
            pub const fn is_create(self) -> bool;
        }
    }
}

/// Holds information about the mouse
#[derive(Default, Debug, Clone)]
#[expect(clippy::struct_excessive_bools, reason = "todo: refactor")]
pub struct SelectionKeysState {
    /// Left mouse click is currently being held down
    pub is_left_down: bool,
    /// Left mouse click is currently being held down
    pub is_right_down: bool,
    /// Shift key is currently being held down
    pub is_shift_down: bool,
    /// Control key is currently being held down
    pub is_ctrl_down: bool,
}
