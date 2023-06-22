use fltk::group::Group;
use fltk::prelude::*;

use super::{LayoutElement, Size};

mod builder;

pub use builder::{CellBuilder, GridBuilder, StripeBuilder};

#[derive(Debug, Clone, Copy)]
pub enum SizingMode {
    Shrink,
    Stretch,
}

#[derive(Debug, Clone, Copy)]
pub enum CellAlign {
    Start,
    Center,
    End,
    Stretch,
}

pub struct Grid {
    props: GridProperties,
    stretch_rows: Vec<usize>,
    stretch_cols: Vec<usize>,
    min_size: Size,
}

struct GridProperties {
    group: Group,
    padding: Padding,
    row_spacing: i32,
    col_spacing: i32,
    cells: Vec<Cell>,
    rows: Vec<Stripe>,
    cols: Vec<Stripe>,
}

#[derive(Debug, Default)]
struct Padding {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

struct Cell {
    element: Box<dyn LayoutElement>,
    min_size: Size,
    props: CellProperties,
}

struct CellProperties {
    row: usize,
    col: usize,
    horz_align: CellAlign,
    vert_align: CellAlign,
}

struct Stripe {
    cells: Vec<StripeCell>,
    min_size: i32,
    props: StripeProperties,
}

struct StripeProperties {
    mode: SizingMode,
}

#[derive(Debug, Clone, Copy)]
enum StripeCell {
    Free,
    Skipped,
    Cell(usize),
}

impl StripeCell {
    fn cell_idx(&self) -> Option<usize> {
        if let Self::Cell(idx) = self {
            Some(*idx)
        } else {
            None
        }
    }
}

impl LayoutElement for Grid {
    fn min_size(&self) -> Size {
        self.min_size
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.props.group.clone().resize(x, y, width, height);
        self.layout_children()
    }
}

impl Grid {
    pub fn builder() -> GridBuilder {
        GridBuilder::new()
    }

    pub fn group(&self) -> Group {
        self.props.group.clone()
    }

    pub fn layout_children(&self) {
        let x = self.props.group.x() + self.props.padding.left;
        let y = self.props.group.y() + self.props.padding.top;
        let width = self.props.group.width() - (self.props.padding.left + self.props.padding.right);
        let height =
            self.props.group.height() - (self.props.padding.top + self.props.padding.bottom);

        let col_bounds = Self::calc_stripe_bounds(
            width,
            &self.props.cols,
            &self.stretch_cols,
            self.props.col_spacing,
        );
        let row_bounds = Self::calc_stripe_bounds(
            height,
            &self.props.rows,
            &self.stretch_rows,
            self.props.row_spacing,
        );

        let mut cell_layouts = Vec::with_capacity(self.props.cells.len());
        for cell in self.props.cells.iter() {
            let (x, width) = col_bounds[cell.props.col];
            let (y, height) = row_bounds[cell.props.row];
            cell_layouts.push((x, y, width, height));
        }

        for (cell_idx, cell) in self.props.cells.iter().enumerate() {
            let (cell_x, cell_y, cell_width, cell_height) = cell_layouts[cell_idx];
            let (widget_x, widget_width) = Self::calc_widget_bounds(
                x,
                cell_x,
                cell_width,
                cell.min_size.width,
                cell.props.horz_align,
            );
            let (widget_y, widget_height) = Self::calc_widget_bounds(
                y,
                cell_y,
                cell_height,
                cell.min_size.height,
                cell.props.vert_align,
            );
            cell.element
                .layout(widget_x, widget_y, widget_width, widget_height);
        }
    }

    fn new(mut props: GridProperties) -> Self {
        let mut min_size = Size {
            width: (props.cols.len() - 1) as i32 * props.col_spacing,
            height: (props.rows.len() - 1) as i32 * props.row_spacing,
        };
        let mut stretch_rows = Vec::new();
        let mut stretch_cols = Vec::new();

        for cell in props.cells.iter_mut() {
            cell.min_size = cell.element.min_size();
        }
        for (col_idx, col) in props.cols.iter_mut().enumerate() {
            col.min_size = col
                .cells
                .iter()
                .filter_map(StripeCell::cell_idx)
                .map(|idx| props.cells[idx].min_size.width)
                .max()
                .unwrap_or_default();
            min_size.width += col.min_size;
            if let SizingMode::Stretch = col.props.mode {
                stretch_cols.push(col_idx);
            }
        }
        for (row_idx, row) in props.rows.iter_mut().enumerate() {
            row.min_size = row
                .cells
                .iter()
                .filter_map(StripeCell::cell_idx)
                .map(|idx| props.cells[idx].min_size.height)
                .max()
                .unwrap_or_default();
            min_size.height += row.min_size;
            if let SizingMode::Stretch = row.props.mode {
                stretch_rows.push(row_idx);
            }
        }

        stretch_rows
            .sort_by(|lidx, ridx| props.rows[*ridx].min_size.cmp(&props.rows[*lidx].min_size));
        stretch_cols
            .sort_by(|lidx, ridx| props.cols[*ridx].min_size.cmp(&props.cols[*lidx].min_size));

        Self {
            props,
            stretch_rows,
            stretch_cols,
            min_size,
        }
    }

    fn calc_stripe_bounds(
        total_size: i32,
        stripes: &[Stripe],
        stretch_stripes: &[usize],
        spacing: i32,
    ) -> Vec<(i32, i32)> {
        let mut bounds = Vec::with_capacity(stripes.len());

        let mut stretch_budget = total_size - (stripes.len() - 1) as i32 * spacing;
        for stripe in stripes.iter() {
            match stripe.props.mode {
                SizingMode::Stretch => (),
                _ => stretch_budget -= stripe.min_size,
            }
            bounds.push((0, stripe.min_size));
        }
        stretch_budget = std::cmp::max(0, stretch_budget);

        let mut stretch_count = stretch_stripes.len() as i32;
        let mut stretch_unit = stretch_budget / stretch_count;
        for &stripe_idx in stretch_stripes.iter() {
            let stripe = &stripes[stripe_idx];

            stretch_count -= 1;
            let stripe_size = if stretch_count > 0 { stretch_unit } else { stretch_budget };

            if stripe_size < stripe.min_size {
                stretch_budget -= stripe.min_size;
                if stretch_count > 0 {
                    stretch_unit = stretch_budget / stretch_count;
                }
            } else {
                stretch_budget -= stripe_size;
                bounds[stripe_idx].1 = stretch_unit;
            }
        }

        let mut start = 0;
        for stripe_bounds in bounds.iter_mut() {
            stripe_bounds.0 = start;
            start += stripe_bounds.1 + spacing;
        }

        bounds
    }

    fn calc_widget_bounds(
        group_start: i32,
        cell_start: i32,
        cell_size: i32,
        min_size: i32,
        align: CellAlign,
    ) -> (i32, i32) {
        let widget_size = match align {
            CellAlign::Stretch => cell_size,
            _ => min_size,
        };
        let widget_start = group_start
            + cell_start
            + match align {
                CellAlign::Start => 0,
                CellAlign::Center => (cell_size - widget_size) / 2,
                CellAlign::End => cell_size - widget_size,
                CellAlign::Stretch => 0,
            };
        (widget_start, widget_size)
    }
}
