//! Grid allows to position items on a canvas in a grid with labels

use bon::Builder;
use iced::{
    Point, Rectangle, Size,
    advanced::graphics::geometry,
    widget::canvas::{Frame, Stroke},
};

use crate::geometry::{PointExt as _, RectangleExt as _, StrokeExt as _, TextExt as _};

/// A cell in a grid
#[derive(Clone, Debug, Builder)]
pub struct Cell<'frame, Draw: FnOnce(&mut Frame, Rectangle)> {
    /// The closure. Determines what to draw inside of the table cell
    draw: Draw,
    /// Stroke to draw around the cell
    stroke: Option<Stroke<'frame>>,
    /// Label of the cell. Drawn above the cell
    label: Option<geometry::Text>,
    /// Description of the cell. Drawn below the cell
    description: Option<geometry::Text>,
}

impl<Draw: FnOnce(&mut Frame, Rectangle)> Cell<'_, Draw> {
    /// Draw the `Cell`
    pub fn draw(self, frame: &mut Frame, bounds: Rectangle) {
        // Stroke
        if let Some(stroke) = self.stroke {
            frame.stroke_rectangle(bounds.top_left(), bounds.size(), stroke);
        }

        // Label
        if let Some(label) = self.label {
            // center horizontally
            let label = label.position(|text_size| {
                Point::new(
                    bounds.center_x_for(text_size),
                    bounds.y - text_size.height - 4.0,
                )
            });

            frame.fill_text(label);
        }

        // Description
        if let Some(description) = self.description {
            let description = description.position(|text_size| {
                Point::new(
                    bounds.center_x_for(text_size),
                    bounds.y + bounds.height + 4.0,
                )
            });

            frame.fill_text(description);
        }

        // Draw cell contents
        (self.draw)(frame, bounds);
    }
}

/// A grid for a canvas
#[derive(Clone, Debug, Builder)]
pub struct Grid<'frame, Draw: FnOnce(&mut Frame, Rectangle)> {
    /// Top-left corner of the `Grid`
    top_left: Point,
    /// Cells of the grid
    cells: Vec<Cell<'frame, Draw>>,
    /// Column count of the grid
    columns: usize,
    /// Size of each item
    cell_size: Size,
    /// Title of the grid. Drawn above the grid
    title: Option<(geometry::Text, f32)>,
    /// Description of the grid. Drawn below the grid
    description: Option<(geometry::Text, f32)>,
    /// Draw red border around grid items, for debugging purposes
    #[builder(default, with = || true)]
    dbg: bool,
    /// How much space to put between each item
    #[builder(default)]
    spacing: Size,
}

impl<Draw: FnOnce(&mut Frame, Rectangle)> Grid<'_, Draw> {
    /// Region occupied by the `Grid`
    pub fn rect(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.size())
    }

    /// Size of the `Grid`
    pub fn size(&self) -> Size {
        let rows = self.cells.len() / self.columns;

        Size {
            width: self.columns as f32 * self.cell_size.width
                + (self.columns as f32 - 1.0) * self.spacing.width,
            height: (rows as f32) * self.cell_size.height
                + (rows as f32 - 1.0) * self.spacing.height
                + self
                    .title
                    .as_ref()
                    .map_or(0.0, |title| title.0.size().height + title.1)
                + self
                    .description
                    .as_ref()
                    .map_or(0.0, |desc| desc.0.size().height + desc.1),
        }
    }

    /// Draw the `Grid` on the `Frame` of a `Canvas`
    pub fn draw(self, frame: &mut Frame) {
        let grid_rect = Rectangle::new(self.top_left, self.size());

        if self.dbg {
            frame.stroke_rectangle(grid_rect.top_left(), grid_rect.size(), Stroke::RED);
        }

        // how much vertical space the title takes up
        let title_vspace = self.title.map_or(0.0, |title| {
            let title_size = title.0.size();

            let text_title = title.0.position(|text_size| Point {
                x: grid_rect.center_x_for(text_size),
                y: grid_rect.y,
            });

            if self.dbg {
                frame.stroke_rectangle(text_title.position, title_size, Stroke::RED);
            }

            frame.fill_text(text_title);

            title_size.height + title.1
        });

        if let Some(desc) = self.description {
            let desc_size = desc.0.size();

            let desc = desc.0.position(|text_size| Point {
                x: grid_rect.center_x_for(text_size),
                y: grid_rect.y + grid_rect.height - text_size.height,
            });

            if self.dbg {
                frame.stroke_rectangle(desc.position, desc_size, Stroke::RED);
            }

            frame.fill_text(desc);
        }

        for (index, cell) in self.cells.into_iter().enumerate() {
            let rows_drawn = (index / self.columns) as f32;
            let cols_drawn = (index % self.columns) as f32;

            let cell_top_left = Point::new(
                cols_drawn * self.cell_size.width + self.spacing.width * cols_drawn,
                rows_drawn * self.cell_size.height
                    + self.spacing.height * rows_drawn
                    + title_vspace,
            ) + self.top_left.into_vector();

            cell.draw(frame, Rectangle::new(cell_top_left, self.cell_size));

            if self.dbg {
                frame.stroke_rectangle(cell_top_left, self.cell_size, Stroke::RED);
            }
        }
    }
}
