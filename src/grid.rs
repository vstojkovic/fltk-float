use std::borrow::Borrow;

use fltk::group::Group;
use fltk::prelude::*;

use crate::WrapperFactory;

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

pub struct Grid<G: GroupExt + Clone = Group> {
    props: GridProperties<G>,
    stretch_rows: Vec<usize>,
    stretch_cols: Vec<usize>,
    min_size: Size,
}

struct GridProperties<G: GroupExt + Clone = Group> {
    group: G,
    padding: Padding,
    row_spacing: i32,
    col_spacing: i32,
    cells: Vec<Cell>,
    spans: Vec<Cell>,
    rows: Vec<Stripe>,
    cols: Vec<Stripe>,
}

#[derive(Debug, Default, Clone, Copy)]
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
    row_span: usize,
    col_span: usize,
    padding: Padding,
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
    Span,
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

impl<G: GroupExt + Clone> LayoutElement for Grid<G> {
    fn min_size(&self) -> Size {
        self.min_size
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.props.group.clone().resize(x, y, width, height);
        self.layout_children()
    }
}

impl Grid {
    pub fn builder() -> GridBuilder<Group, WrapperFactory> {
        GridBuilder::new(Group::default_fill())
    }

    pub fn builder_with_factory<F: Borrow<WrapperFactory>>(factory: F) -> GridBuilder<Group, F> {
        GridBuilder::with_factory(Group::default_fill(), factory)
    }
}

impl<G: GroupExt + Clone> Grid<G> {
    pub fn group(&self) -> G {
        self.props.group.clone()
    }

    pub fn layout_children(&self) {
        let x = self.props.group.x() + self.props.padding.left;
        let y = self.props.group.y() + self.props.padding.top;
        let width = self.props.group.width() - (self.props.padding.left + self.props.padding.right);
        let height =
            self.props.group.height() - (self.props.padding.top + self.props.padding.bottom);

        // TODO: Eliminate unnecessary allocation
        let col_bounds = calc_stripe_bounds(
            width,
            &self.props.cols,
            &self.stretch_cols,
            self.props.col_spacing,
        );
        let row_bounds = calc_stripe_bounds(
            height,
            &self.props.rows,
            &self.stretch_rows,
            self.props.row_spacing,
        );

        for cell in self.props.cells.iter() {
            let (cell_x, cell_width) = col_bounds[cell.props.col];
            let (cell_y, cell_height) = row_bounds[cell.props.row];
            let (widget_x, widget_width) = calc_widget_bounds(
                x,
                cell_x,
                cell_width,
                cell.min_size.width,
                cell.props.padding.left,
                cell.props.padding.right,
                cell.props.horz_align,
            );
            let (widget_y, widget_height) = calc_widget_bounds(
                y,
                cell_y,
                cell_height,
                cell.min_size.height,
                cell.props.padding.top,
                cell.props.padding.bottom,
                cell.props.vert_align,
            );
            cell.element
                .layout(widget_x, widget_y, widget_width, widget_height);
        }

        for span in self.props.spans.iter() {
            let left_col = span.props.col;
            let right_col = left_col + span.props.col_span - 1;
            let span_x = col_bounds[left_col].0;
            let span_width = col_bounds[right_col].0 + col_bounds[right_col].1 - span_x;

            let top_row = span.props.row;
            let bottom_row = top_row + span.props.row_span - 1;
            let span_y = row_bounds[top_row].0;
            let span_height = row_bounds[bottom_row].0 + row_bounds[bottom_row].1 - span_y;

            let (widget_x, widget_width) = calc_widget_bounds(
                x,
                span_x,
                span_width,
                span.min_size.width,
                span.props.padding.left,
                span.props.padding.right,
                span.props.horz_align,
            );
            let (widget_y, widget_height) = calc_widget_bounds(
                y,
                span_y,
                span_height,
                span.min_size.height,
                span.props.padding.top,
                span.props.padding.bottom,
                span.props.vert_align,
            );
            span.element
                .layout(widget_x, widget_y, widget_width, widget_height);
        }
    }

    fn new(props: GridProperties<G>) -> Self {
        let stretch_rows = collect_stretch_stripes(&props.rows);
        let stretch_cols = collect_stretch_stripes(&props.cols);

        let mut grid = Self {
            props,
            stretch_rows,
            stretch_cols,
            min_size: Default::default(),
        };

        grid.cache_min_sizes();

        sort_stretch_stripes(&grid.props.rows, &mut grid.stretch_rows);
        sort_stretch_stripes(&grid.props.cols, &mut grid.stretch_cols);

        grid
    }

    fn cache_min_sizes(&mut self) {
        self.cache_cell_min_sizes();
        self.cache_span_min_sizes();

        self.min_size.width = span_size(&self.props.cols, self.props.col_spacing)
            + self.props.padding.left
            + self.props.padding.right;
        self.min_size.height = span_size(&self.props.rows, self.props.row_spacing)
            + self.props.padding.top
            + self.props.padding.bottom;
    }

    fn cache_cell_min_sizes(&mut self) {
        for cell in self.props.cells.iter_mut() {
            cell.cache_min_size();
        }
        for col in self.props.cols.iter_mut() {
            col.min_size = col
                .cells
                .iter()
                .filter_map(StripeCell::cell_idx)
                .map(|idx| self.props.cells[idx].min_size.width)
                .max()
                .unwrap_or_default();
        }
        for row in self.props.rows.iter_mut() {
            row.min_size = row
                .cells
                .iter()
                .filter_map(StripeCell::cell_idx)
                .map(|idx| self.props.cells[idx].min_size.height)
                .max()
                .unwrap_or_default();
        }
    }

    fn cache_span_min_sizes(&mut self) {
        for span in self.props.spans.iter_mut() {
            span.cache_min_size();

            let top = span.props.row;
            let bottom = top + span.props.row_span;
            let left = span.props.col;
            let right = left + span.props.col_span;

            adjust_span_stripes(
                span.min_size.width,
                &mut self.props.cols[left..right],
                self.props.col_spacing,
            );
            adjust_span_stripes(
                span.min_size.height,
                &mut self.props.rows[top..bottom],
                self.props.row_spacing,
            );
        }
    }
}

impl Cell {
    fn cache_min_size(&mut self) {
        self.min_size = self.element.min_size();
        self.min_size.width += self.props.padding.left + self.props.padding.right;
        self.min_size.height += self.props.padding.top + self.props.padding.bottom;
    }
}

fn collect_stretch_stripes(stripes: &[Stripe]) -> Vec<usize> {
    stripes
        .iter()
        .enumerate()
        .filter_map(
            |(idx, stripe)| {
                if let SizingMode::Stretch = stripe.props.mode {
                    Some(idx)
                } else {
                    None
                }
            },
        )
        .collect()
}

fn sort_stretch_stripes(stripes: &[Stripe], stretch_stripes: &mut [usize]) {
    stretch_stripes.sort_by(|lidx, ridx| stripes[*ridx].min_size.cmp(&stripes[*lidx].min_size));
}

fn span_size(stripes: &[Stripe], spacing: i32) -> i32 {
    if stripes.len() == 0 {
        return 0;
    }

    let mut size = stripes.iter().map(|stripe| stripe.min_size).sum();
    size += (stripes.len() as i32 - 1) * spacing;
    size
}

fn adjust_span_stripes(min_size: i32, stripes: &mut [Stripe], spacing: i32) {
    let current_size = span_size(stripes, spacing);
    if current_size >= min_size {
        return;
    }

    let mut stretch_stripes = collect_stretch_stripes(stripes);
    if stretch_stripes.len() > 0 {
        sort_stretch_stripes(stripes, &mut stretch_stripes);
        let bounds = calc_stripe_bounds(min_size, stripes, &stretch_stripes, spacing);
        for idx in stretch_stripes {
            stripes[idx].min_size = bounds[idx].1;
        }
    } else {
        stripes[0].min_size += min_size - current_size;
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
    let mut stretch_unit = if stretch_count > 0 { stretch_budget / stretch_count } else { 0 };
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
    pad_start: i32,
    pad_end: i32,
    align: CellAlign,
) -> (i32, i32) {
    let widget_size = match align {
        CellAlign::Stretch => cell_size,
        _ => min_size,
    };

    let widget_size = widget_size - pad_start - pad_end;
    let cell_size = cell_size - pad_start - pad_end;

    let widget_start = match align {
        CellAlign::Start => 0,
        CellAlign::Center => (cell_size - widget_size) / 2,
        CellAlign::End => cell_size - widget_size,
        CellAlign::Stretch => 0,
    };
    let widget_start = group_start + pad_start + cell_start + widget_start;

    (widget_start, widget_size)
}
