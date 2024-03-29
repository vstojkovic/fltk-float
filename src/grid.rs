use std::borrow::Borrow;
use std::rc::Rc;

use fltk::group::Group;
use fltk::prelude::*;

use crate::WrapperFactory;

use super::{LayoutElement, Padding, Size};

mod builder;

pub use builder::{CellBuilder, GridBuilder, StripeBuilder};

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
    groups: Vec<StripeProperties>,
    rows: Vec<Stripe>,
    cols: Vec<Stripe>,
}

struct Cell {
    element: Rc<dyn LayoutElement>,
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

#[derive(Debug, Clone, Copy)]
struct StripeProperties {
    stretch: u8,
    min_size: i32,
}

struct Stripe {
    cells: Vec<StripeCell>,
    group_idx: usize,
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
            &self.props.groups,
            &self.stretch_cols,
            self.props.col_spacing,
        );
        let row_bounds = calc_stripe_bounds(
            height,
            &self.props.rows,
            &self.props.groups,
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
        let stretch_rows = collect_stretch_stripes(&props.rows, &props.groups);
        let stretch_cols = collect_stretch_stripes(&props.cols, &props.groups);

        let mut grid = Self {
            props,
            stretch_rows,
            stretch_cols,
            min_size: Default::default(),
        };

        grid.cache_min_sizes();

        sort_stretch_stripes(&grid.props.rows, &grid.props.groups, &mut grid.stretch_rows);
        sort_stretch_stripes(&grid.props.cols, &grid.props.groups, &mut grid.stretch_cols);

        grid
    }

    fn cache_min_sizes(&mut self) {
        self.cache_cell_min_sizes();
        self.cache_span_min_sizes();

        self.min_size.width =
            span_size(&self.props.cols, &self.props.groups, self.props.col_spacing)
                + self.props.padding.left
                + self.props.padding.right;
        self.min_size.height =
            span_size(&self.props.rows, &self.props.groups, self.props.row_spacing)
                + self.props.padding.top
                + self.props.padding.bottom;
    }

    fn cache_cell_min_sizes(&mut self) {
        for cell in self.props.cells.iter_mut() {
            cell.cache_min_size();
        }
        for col in self.props.cols.iter_mut() {
            self.props.groups[col.group_idx].min_size = col
                .cells
                .iter()
                .filter_map(StripeCell::cell_idx)
                .map(|idx| self.props.cells[idx].min_size.width)
                .fold(self.props.groups[col.group_idx].min_size, std::cmp::max);
        }
        for row in self.props.rows.iter_mut() {
            self.props.groups[row.group_idx].min_size = row
                .cells
                .iter()
                .filter_map(StripeCell::cell_idx)
                .map(|idx| self.props.cells[idx].min_size.height)
                .fold(self.props.groups[row.group_idx].min_size, std::cmp::max);
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
                &self.props.cols[left..right],
                &mut self.props.groups,
                self.props.col_spacing,
            );
            adjust_span_stripes(
                span.min_size.height,
                &self.props.rows[top..bottom],
                &mut self.props.groups,
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

fn collect_stretch_stripes(stripes: &[Stripe], groups: &[StripeProperties]) -> Vec<usize> {
    stripes
        .iter()
        .enumerate()
        .filter_map(
            |(idx, stripe)| {
                if groups[stripe.group_idx].stretch > 0 {
                    Some(idx)
                } else {
                    None
                }
            },
        )
        .collect()
}

fn sort_stretch_stripes(
    stripes: &[Stripe],
    groups: &[StripeProperties],
    stretch_stripes: &mut [usize],
) {
    stretch_stripes.sort_by(|lidx, ridx| {
        groups[stripes[*ridx].group_idx]
            .min_size
            .cmp(&groups[stripes[*lidx].group_idx].min_size)
    });
}

fn span_size(stripes: &[Stripe], groups: &[StripeProperties], spacing: i32) -> i32 {
    if stripes.len() == 0 {
        return 0;
    }

    let mut size = stripes
        .iter()
        .map(|stripe| groups[stripe.group_idx].min_size)
        .sum();
    size += (stripes.len() as i32 - 1) * spacing;
    size
}

fn adjust_span_stripes(
    min_size: i32,
    stripes: &[Stripe],
    groups: &mut [StripeProperties],
    spacing: i32,
) {
    let current_size = span_size(stripes, groups, spacing);
    if current_size >= min_size {
        return;
    }

    let mut stretch_stripes = collect_stretch_stripes(stripes, groups);
    if stretch_stripes.len() > 0 {
        sort_stretch_stripes(stripes, groups, &mut stretch_stripes);
        let bounds = calc_stripe_bounds(min_size, stripes, groups, &stretch_stripes, spacing);
        for idx in stretch_stripes {
            groups[stripes[idx].group_idx].min_size = bounds[idx].1;
        }
    } else {
        groups[stripes[0].group_idx].min_size += min_size - current_size;
    }
}

fn calc_stripe_bounds(
    total_size: i32,
    stripes: &[Stripe],
    groups: &[StripeProperties],
    stretch_stripes: &[usize],
    spacing: i32,
) -> Vec<(i32, i32)> {
    let mut bounds = Vec::with_capacity(stripes.len());

    let mut stretch_budget = total_size - (stripes.len() - 1) as i32 * spacing;
    let mut stretch_count: i32 = 0;
    for stripe in stripes.iter() {
        let group = &groups[stripe.group_idx];
        if group.stretch == 0 {
            stretch_budget -= group.min_size;
        } else {
            stretch_count += group.stretch as i32;
        }
        bounds.push((0, group.min_size));
    }
    stretch_budget = std::cmp::max(0, stretch_budget);

    let mut stretch_unit = if stretch_count > 0 { stretch_budget / stretch_count } else { 0 };
    for &stripe_idx in stretch_stripes.iter() {
        let stripe = &stripes[stripe_idx];
        let group = &groups[stripe.group_idx];

        let factor = group.stretch as i32;
        stretch_count -= factor;
        let stripe_size = if stretch_count > 0 { stretch_unit * factor } else { stretch_budget };

        if stripe_size < group.min_size {
            stretch_budget -= group.min_size;
            if stretch_count > 0 {
                stretch_unit = stretch_budget / stretch_count;
            }
        } else {
            stretch_budget -= stripe_size;
            bounds[stripe_idx].1 = stripe_size;
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
