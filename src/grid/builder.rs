use std::borrow::Borrow;

use fltk::group::Group;
use fltk::prelude::*;

use crate::WrapperFactory;

use super::{Cell, Grid, GridProperties, Padding, StripeCell};

mod cell;
mod stripe;

pub use cell::CellBuilder;
pub use stripe::StripeBuilder;

pub struct GridBuilder<G: GroupExt + Clone = Group, F: Borrow<WrapperFactory> = WrapperFactory> {
    props: GridProperties<G>,
    factory: F,
    default_cell_padding: Padding,
    next_row: usize,
    next_col: usize,
}

impl<G: GroupExt + Clone> GridBuilder<G, WrapperFactory> {
    pub fn new(group: G) -> Self {
        Self::with_factory(group, WrapperFactory::new())
    }
}

impl<G: GroupExt + Clone, F: Borrow<WrapperFactory>> GridBuilder<G, F> {
    pub fn with_factory(group: G, factory: F) -> Self {
        Self {
            props: GridProperties {
                group,
                padding: Default::default(),
                row_spacing: 0,
                col_spacing: 0,
                cells: Vec::new(),
                spans: Vec::new(),
                rows: Vec::new(),
                cols: Vec::new(),
            },
            factory,
            default_cell_padding: Default::default(),
            next_row: 0,
            next_col: 0,
        }
    }

    pub fn with_row_spacing(mut self, spacing: i32) -> Self {
        self.props.row_spacing = std::cmp::max(0, spacing);
        self
    }

    pub fn with_col_spacing(mut self, spacing: i32) -> Self {
        self.props.col_spacing = std::cmp::max(0, spacing);
        self
    }

    pub fn with_left_padding(mut self, padding: i32) -> Self {
        self.props.padding.left = padding;
        self
    }

    pub fn with_top_padding(mut self, padding: i32) -> Self {
        self.props.padding.top = padding;
        self
    }

    pub fn with_right_padding(mut self, padding: i32) -> Self {
        self.props.padding.right = padding;
        self
    }

    pub fn with_bottom_padding(mut self, padding: i32) -> Self {
        self.props.padding.bottom = padding;
        self
    }

    pub fn with_padding(mut self, left: i32, top: i32, right: i32, bottom: i32) -> Self {
        self.props.padding = Padding {
            left,
            top,
            right,
            bottom,
        };
        self
    }

    pub fn with_default_cell_padding(
        mut self,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) -> Self {
        self.default_cell_padding = Padding {
            left,
            top,
            right,
            bottom,
        };
        self
    }

    pub fn row(&mut self) -> StripeBuilder<G, F> {
        StripeBuilder::new_row(self)
    }

    pub fn col(&mut self) -> StripeBuilder<G, F> {
        StripeBuilder::new_col(self)
    }

    pub fn cell(&mut self) -> Option<CellBuilder<G, F>> {
        let (row, col) = self.next_free_cell()?;
        Some(CellBuilder::new(self, row, col, 1, 1))
    }

    pub fn cell_at(&mut self, row: usize, col: usize) -> Option<CellBuilder<G, F>> {
        if (row >= self.props.rows.len()) && (col >= self.props.cols.len()) {
            return None;
        }
        match self.props.rows[row].cells[col] {
            StripeCell::Free | StripeCell::Skipped => Some(CellBuilder::new(self, row, col, 1, 1)),
            _ => None,
        }
    }

    pub fn span(&mut self, row_span: usize, col_span: usize) -> Option<CellBuilder<G, F>> {
        if (row_span == 0) || (col_span == 0) {
            return None;
        }

        let (row, col) = self.next_free_cell()?;
        if self.is_span_available(row, col, row_span, col_span, false) {
            Some(CellBuilder::new(self, row, col, row_span, col_span))
        } else {
            None
        }
    }

    pub fn span_at(
        &mut self,
        row: usize,
        col: usize,
        row_span: usize,
        col_span: usize,
    ) -> Option<CellBuilder<G, F>> {
        if (row_span == 0) || (col_span == 0) {
            return None;
        }

        if (row >= self.props.rows.len()) && (col >= self.props.cols.len()) {
            return None;
        }
        if self.is_span_available(row, col, row_span, col_span, true) {
            Some(CellBuilder::new(self, row, col, row_span, col_span))
        } else {
            None
        }
    }

    pub fn end(self) -> Grid<G> {
        self.props.group.end();
        Grid::new(self.props)
    }

    fn next_free_cell(&mut self) -> Option<(usize, usize)> {
        let mut row = self.next_row;
        let mut col = self.next_col;

        loop {
            if col >= self.props.cols.len() {
                col = 0;
                row += 1;
            }
            if row >= self.props.rows.len() {
                return None;
            }
            if let StripeCell::Free = self.props.rows[row].cells[col] {
                break;
            }
            col += 1;
        }

        self.next_row = row;
        self.next_col = col;

        Some((row, col))
    }

    fn is_span_available(
        &self,
        row: usize,
        col: usize,
        row_span: usize,
        col_span: usize,
        allow_skipped: bool,
    ) -> bool {
        let top = row;
        let bottom = top + row_span;
        let left = col;
        let right = left + col_span;

        if (bottom > self.props.rows.len()) || (right > self.props.cols.len()) {
            return false;
        }

        for cell_row in top..bottom {
            for cell_col in left..right {
                let cell_available = match self.props.rows[cell_row].cells[cell_col] {
                    StripeCell::Free => true,
                    StripeCell::Skipped if allow_skipped => true,
                    _ => false,
                };
                if !cell_available {
                    return false;
                }
            }
        }

        true
    }

    fn add_cell(&mut self, cell: Cell) {
        if (cell.props.row_span > 1) || (cell.props.col_span > 1) {
            return self.add_span(cell);
        }

        let row = cell.props.row;
        let col = cell.props.col;

        let cell_idx = self.props.cells.len();
        self.props.cells.push(cell);

        self.props.rows[row].cells[col] = StripeCell::Cell(cell_idx);
        self.props.cols[col].cells[row] = StripeCell::Cell(cell_idx);
    }

    fn add_span(&mut self, span: Cell) {
        let top = span.props.row;
        let bottom = top + span.props.row_span;
        let left = span.props.col;
        let right = left + span.props.col_span;

        self.props.spans.push(span);

        for row in top..bottom {
            for col in left..right {
                self.props.rows[row].cells[col] = StripeCell::Span;
                self.props.cols[col].cells[row] = StripeCell::Span;
            }
        }
    }
}
