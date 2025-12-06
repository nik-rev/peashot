//! Keybindings cheatsheet

use iced::{
    Background, Element, Font,
    Length::Fill,
    Pixels, Point, Rectangle, Renderer, Size, Task, Theme, Vector,
    advanced::{graphics::geometry, svg::Svg},
    font::{self, Family, Weight},
    widget::{
        canvas,
        canvas::{LineCap, LineJoin, Stroke},
        column, container,
        text::Shaping,
    },
};

use crate::{
    geometry::{PointExt as _, RectangleExt as _, SizeExt as _, VectorExt as _},
    icons::Icon,
    ui::{grid::Grid, selection::Selection},
};

use super::Popup;

crate::declare_commands! {
    enum Command {
        /// Open the keybindings cheatsheet
        OpenKeybindingsCheatsheet,
    }
}

impl crate::command::Handler for Command {
    fn handle(self, app: &mut crate::App, _count: u32) -> Task<crate::Message> {
        match self {
            Self::OpenKeybindingsCheatsheet => {
                app.popup = Some(Popup::KeyCheatsheet);
            }
        }

        Task::none()
    }
}

/// Keybindings cheatsheet message
#[derive(Debug, Clone)]
pub enum Message {
    /// Open the keybindings menu
    Open,
    /// Close the keybindings menu
    Close,
}

impl crate::message::Handler for Message {
    fn handle(self, app: &mut crate::App) -> Task<crate::Message> {
        match self {
            Self::Open => app.popup = Some(Popup::KeyCheatsheet),
            Self::Close => app.popup = None,
        }

        Task::none()
    }
}

/// Show a cheatsheet for the default keybindings available in ferrishot
#[derive(Debug, Copy, Clone)]
pub struct KeybindingsCheatsheet<'app> {
    /// Theme of the app
    pub theme: &'app crate::Theme,
}

impl<'app> KeybindingsCheatsheet<'app> {
    /// Show the keybinding cheatsheet
    pub fn view(self) -> Element<'app, crate::Message> {
        let size = Size::new(1550.0, 1000.0);
        super::popup(
            size,
            container(column![canvas(self).width(Fill).height(Fill)])
                .style(|_| container::Style {
                    background: Some(Background::Color(self.theme.cheatsheet_bg)),
                    ..Default::default()
                })
                .width(size.width)
                .height(size.height),
            self.theme,
        )
    }
}

/// Applies a transformation to the old selection, yielding the new selection
/// after some movement
type SelectionTransformer =
    fn(origin: Point, sel_size: Size, cell_size: Size, old_sel: Selection) -> Selection;

/// Cell definiton
type CellDefinition<'a> = (
    // keybinding
    &'a str,
    // label
    &'a str,
    // Compute the new selection
    fn(Selection) -> Selection,
    (
        Icon,
        // Compute position of icon
        fn(Selection) -> Point,
    ),
);

impl canvas::Program<crate::Message> for KeybindingsCheatsheet<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        /// How far away the `new` selection from the `old` selection should be
        const SEL_NEW_OLD_OFFSET: f32 = 20.0;
        /// Size of each arrow
        const ARROW_ICON_SIZE: f32 = 18.0;

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let theme_with_dimmed_sel = crate::Theme {
            selection_frame: self.theme.selection_frame.scale_alpha(0.3),
            ..*self.theme
        };

        let cell_definitions: [CellDefinition; 12] = [
            (
                "h or 🡰",
                "Nudge Left",
                |sel| sel.with_x(|x| x - SEL_NEW_OLD_OFFSET),
                (Icon::ArrowLeft, |new_sel| {
                    new_sel.center() - Vector::diag(ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "l or 🡲",
                "Nudge Right",
                |sel| sel.with_x(|x| x + SEL_NEW_OLD_OFFSET),
                (Icon::ArrowRight, |new_sel| {
                    new_sel.center() - Vector::diag(ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "j or 🡳",
                "Nudge Down",
                |sel| sel.with_y(|y| y + SEL_NEW_OLD_OFFSET),
                (Icon::ArrowDown, |new_sel| {
                    new_sel.center() - Vector::diag(ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "k or 🡱",
                "Nudge Up",
                |sel| sel.with_y(|y| y - SEL_NEW_OLD_OFFSET),
                (Icon::ArrowUp, |new_sel| {
                    new_sel.center() - Vector::diag(ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "shift h or 🡰",
                "Extend Left",
                |sel| {
                    sel.with_x(|x| x - SEL_NEW_OLD_OFFSET)
                        .with_width(|w| w + SEL_NEW_OLD_OFFSET)
                },
                (Icon::ArrowLeft, |new_sel| {
                    new_sel
                        .left_center()
                        .with_y(|y| y - ARROW_ICON_SIZE / 2.0)
                        .with_x(|x| x - ARROW_ICON_SIZE)
                }),
            ),
            (
                "shift l or 🡲",
                "Extend Right",
                |sel| sel.with_width(|w| w + SEL_NEW_OLD_OFFSET),
                (Icon::ArrowRight, |new_sel| {
                    new_sel.right_center().with_y(|y| y - ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "shift j or 🡳",
                "Extend Bottom",
                |sel| sel.with_height(|h| h + SEL_NEW_OLD_OFFSET),
                (Icon::ArrowDown, |new_sel| {
                    new_sel
                        .bottom_center()
                        .with_x(|x| x - ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "shift k or 🡱",
                "Extend Top",
                |sel| {
                    sel.with_y(|y| y - SEL_NEW_OLD_OFFSET)
                        .with_height(|h| h + SEL_NEW_OLD_OFFSET)
                },
                (Icon::ArrowUp, |new_sel| {
                    new_sel
                        .top_center()
                        .with_x(|x| x - ARROW_ICON_SIZE / 2.0)
                        .with_y(|y| y - ARROW_ICON_SIZE)
                }),
            ),
            (
                "ctrl l or 🡲",
                "Shrink Left",
                |sel| {
                    sel.with_x(|x| x + SEL_NEW_OLD_OFFSET)
                        .with_width(|w| w - SEL_NEW_OLD_OFFSET)
                },
                (Icon::ArrowRight, |new_sel| {
                    new_sel.left_center().with_y(|y| y - ARROW_ICON_SIZE / 2.0)
                }),
            ),
            (
                "ctrl h or 🡰",
                "Shrink Right",
                |sel| sel.with_width(|w| w - SEL_NEW_OLD_OFFSET),
                (Icon::ArrowLeft, |new_sel| {
                    new_sel
                        .right_center()
                        .with_y(|y| y - ARROW_ICON_SIZE / 2.0)
                        .with_x(|x| x - ARROW_ICON_SIZE)
                }),
            ),
            (
                "ctrl j or 🡳",
                "Shrink Down",
                |sel| sel.with_height(|h| h - SEL_NEW_OLD_OFFSET),
                (Icon::ArrowUp, |new_sel| {
                    new_sel
                        .bottom_center()
                        .with_x(|x| x - ARROW_ICON_SIZE / 2.0)
                        .with_y(|y| y - ARROW_ICON_SIZE)
                }),
            ),
            (
                "ctrl k or 🡱",
                "Shrink Up",
                |sel| {
                    sel.with_y(|y| y + SEL_NEW_OLD_OFFSET)
                        .with_height(|h| h - SEL_NEW_OLD_OFFSET)
                },
                (Icon::ArrowDown, |new_sel| {
                    new_sel.top_center().with_x(|x| x - ARROW_ICON_SIZE / 2.0)
                }),
            ),
        ];

        let cells = cell_definitions
            .into_iter()
            .map(|(key, label, compute_new_sel, (icon, icon_pos_fn))| {
                crate::ui::grid::Cell::builder()
                    .draw(move |frame: &mut canvas::Frame, bounds: Rectangle| {
                        let sel_size = 100.0;
                        let old_sel = Selection::new(
                            bounds.center() - Vector::diag(sel_size / 2.0),
                            &theme_with_dimmed_sel,
                            false,
                            None,
                        )
                        .with_size(|_| Size::square(sel_size));

                        let new_sel = compute_new_sel(old_sel).with_theme(self.theme);

                        let icon_pos_relative = icon_pos_fn(new_sel);

                        // draw selection BEFORE transformation
                        old_sel.draw_border(frame);

                        // draw the arrow
                        frame.draw_svg(
                            Rectangle {
                                x: icon_pos_relative.x,
                                y: icon_pos_relative.y,
                                width: ARROW_ICON_SIZE,
                                height: ARROW_ICON_SIZE,
                            },
                            Svg::new(icon.svg()).color(self.theme.cheatsheet_fg),
                        );

                        // draw selection AFTER transformation
                        new_sel.draw_border(frame);
                        new_sel.draw_corners(frame);
                    })
                    .label(canvas::Text {
                        content: key.to_string(),
                        color: self.theme.cheatsheet_fg,
                        font: Font::MONOSPACE,
                        shaping: Shaping::Advanced,
                        ..Default::default()
                    })
                    .description(canvas::Text {
                        content: label.to_string(),
                        color: self.theme.selection_frame,
                        font: Font {
                            family: Family::Monospace,
                            weight: Weight::Normal,
                            style: font::Style::Italic,
                            ..Default::default()
                        },
                        shaping: Shaping::Advanced,
                        ..Default::default()
                    })
                    .build()
            })
            .collect::<Vec<_>>();

        let basic_bindings = Grid::builder()
            .top_left(Point::new(60.0, 0.0))
            .cell_size(Size::new(100.0, 160.0))
            .spacing(Size::new(100.0, 140.0))
            .columns(4)
            .title((
                geometry::Text {
                    content: "Transform region by 1px:".to_string(),
                    color: self.theme.cheatsheet_fg,
                    font: Font::MONOSPACE,
                    size: Pixels(30.0),
                    shaping: Shaping::Advanced,
                    ..Default::default()
                },
                75.0,
            ))
            .description((
                geometry::Text {
                    content: "Hold ALT while doing any of the above to transform by 125px!"
                        .to_string(),
                    color: self.theme.cheatsheet_fg,
                    size: Pixels(20.0),
                    font: Font::MONOSPACE,
                    shaping: Shaping::Advanced,
                    ..Default::default()
                },
                75.0,
            ))
            .cells(cells)
            .build();

        let basic_bindings_size = basic_bindings.size();

        basic_bindings.draw(&mut frame);

        let region_movement_bindings_data: &[(&str, &str, SelectionTransformer)] = &[
            (
                "gk or g🡱",
                "go up as far\nas possible",
                |origin, _, _, old_sel| old_sel.with_y(|_| origin.y),
            ),
            (
                "gj or g🡳",
                "go down as far\nas possible",
                |origin, sel_size, cell_size, old_sel| {
                    old_sel.with_y(|_| origin.y + cell_size.height - sel_size.height)
                },
            ),
            (
                "gl or g🡲",
                "go right as far\nas possible",
                |origin, sel_size, cell_size, old_sel| {
                    old_sel.with_x(|_| origin.x + cell_size.width - sel_size.width)
                },
            ),
            (
                "gh or g🡰",
                "go left as far\nas possible",
                |origin, _, _, old_sel| old_sel.with_x(|_| origin.x),
            ),
            (
                "gx",
                "go to x-center",
                |origin, sel_size, cell_size, old_sel| {
                    old_sel.with_x(|_| origin.x + cell_size.width / 2.0 - sel_size.width / 2.0)
                },
            ),
            (
                "gy",
                "go to y-center",
                |origin, sel_size, cell_size, old_sel| {
                    old_sel.with_y(|_| origin.y + cell_size.height / 2.0 - sel_size.height / 2.0)
                },
            ),
            (
                "gc",
                "go to center",
                |origin, sel_size, cell_size, old_sel| {
                    old_sel.with_pos(|_| {
                        Point::new(
                            cell_size.width / 2.0 - sel_size.width / 2.0,
                            cell_size.height / 2.0 - sel_size.height / 2.0,
                        ) + origin.into_vector()
                    })
                },
            ),
            ("gg", "go to top left", |origin, _, _, old_sel| {
                old_sel.with_pos(|_| origin)
            }),
            (
                "G",
                "go to bottom right",
                |origin, sel_size, cell_size, old_sel| {
                    old_sel.with_pos(|_| {
                        Point::new(
                            cell_size.width - sel_size.width,
                            cell_size.height - sel_size.height,
                        ) + origin.into_vector()
                    })
                },
            ),
        ];

        let region_movement_bindings = Grid::builder()
            .top_left(Point::new(basic_bindings_size.width + 270.0, 0.0))
            .cell_size(Size::new(100.0, 100.0))
            .spacing(Size::new(90.0, 100.0))
            .title((
                canvas::Text {
                    content: "Move region:".to_string(),
                    size: 30.0.into(),
                    color: self.theme.cheatsheet_fg,
                    font: Font::MONOSPACE,
                    shaping: Shaping::Advanced,
                    ..Default::default()
                },
                75.0,
            ))
            .columns(3)
            .cells(
                region_movement_bindings_data
                    .iter()
                    .map(|(key, desc, transform_old_sel)| {
                        crate::ui::grid::Cell::builder()
                            .draw(move |frame: &mut canvas::Frame, bounds: Rectangle| {
                                let cell_size = Size::new(100.0, 100.0);
                                let sel_size = Size::square(40.0);
                                let origin = bounds.top_left();

                                let old_pos = Point::new(
                                    (-1.5f32) * sel_size.width + cell_size.width,
                                    0.5 * sel_size.height,
                                ) + origin.into_vector();

                                let old_sel =
                                    Selection::new(old_pos, &theme_with_dimmed_sel, false, None)
                                        .with_size(|_| sel_size);

                                old_sel.draw_border(frame);

                                let new_sel =
                                    transform_old_sel(origin, sel_size, cell_size, old_sel)
                                        .with_theme(self.theme);

                                new_sel.draw_border(frame);
                                new_sel.draw_corners(frame);
                            })
                            .stroke(Stroke {
                                style: geometry::Style::Solid(self.theme.cheatsheet_fg),
                                width: 1.0,
                                line_cap: LineCap::Round,
                                line_join: LineJoin::Round,
                                line_dash: canvas::LineDash {
                                    segments: &[10.0],
                                    offset: 0,
                                },
                            })
                            .label(canvas::Text {
                                content: (*key).to_string(),
                                color: self.theme.cheatsheet_fg,
                                font: Font::MONOSPACE,
                                shaping: Shaping::Advanced,
                                ..Default::default()
                            })
                            .description(canvas::Text {
                                content: (*desc).to_string(),
                                color: self.theme.selection_frame,
                                font: Font {
                                    family: Family::Monospace,
                                    weight: Weight::Normal,
                                    style: font::Style::Italic,
                                    ..Default::default()
                                },
                                shaping: Shaping::Advanced,
                                ..Default::default()
                            })
                            .build()
                    })
                    .collect::<Vec<_>>(),
            )
            .build();

        let region_movement_bindings_rect = region_movement_bindings.rect();
        region_movement_bindings.draw(&mut frame);

        Grid::builder()
            .top_left(region_movement_bindings_rect.bottom_left() + Vector::y(60.0))
            .title((
                canvas::Text {
                    content: "Pick top and then bottom corners".into(),
                    color: self.theme.cheatsheet_fg,
                    size: Pixels(30.0),
                    font: Font::MONOSPACE,
                    ..Default::default()
                },
                15.0,
            ))
            .description((
                canvas::Text {
                    content: "select any area of the screen in 8 keystrokes!".into(),
                    color: self.theme.cheatsheet_fg,
                    size: Pixels(20.0),
                    font: Font::MONOSPACE,
                    ..Default::default()
                },
                15.0,
            ))
            .cell_size(Size::new(region_movement_bindings_rect.size().width, 200.0))
            .columns(1)
            .cells(vec![
                crate::ui::grid::Cell::builder()
                    .draw(|frame, cell_rect| {
                        let sel_size = Size::square(100.0);

                        let sel =
                            Selection::new(cell_rect.center_for(sel_size), self.theme, false, None)
                                .with_size(|_| sel_size);

                        sel.draw_border(frame);
                        sel.draw_corners(frame);

                        let dotted_stroke = Stroke {
                            style: canvas::Style::Solid(self.theme.selection_frame),
                            width: 3.0,
                            line_cap: LineCap::Round,
                            line_join: LineJoin::Round,
                            line_dash: canvas::LineDash {
                                segments: &[5.0],
                                offset: 0,
                            },
                        };

                        let radius = 25.0;

                        frame.stroke(
                            &geometry::Path::circle(sel.top_left(), radius),
                            dotted_stroke,
                        );
                        frame.stroke(
                            &geometry::Path::circle(sel.bottom_right(), radius),
                            dotted_stroke,
                        );

                        // top left label
                        frame.fill_text(canvas::Text {
                            content: "Pick top left corner: t".into(),
                            position: sel.top_left() - Vector::new(200.0, 20.0),
                            color: self.theme.cheatsheet_fg,
                            ..Default::default()
                        });

                        // bottom right label
                        frame.fill_text(canvas::Text {
                            content: "Pick bottom right corner: b".into(),
                            position: sel.bottom_right() + Vector::x(50.0),
                            color: self.theme.cheatsheet_fg,
                            ..Default::default()
                        });
                    })
                    .build(),
            ])
            .build()
            .draw(&mut frame);

        vec![frame.into_geometry()]
    }
}
