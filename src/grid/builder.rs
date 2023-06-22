use fltk::group::Group;
use fltk::prelude::*;

use super::{Grid, GridProperties, Padding, StripeCell};

mod cell;
mod stripe;

pub use cell::CellBuilder;
pub use stripe::StripeBuilder;

pub struct GridBuilder {
    props: GridProperties,
    next_row: usize,
    next_col: usize,
}

impl GridBuilder {
    pub fn new() -> Self {
        Self {
            props: GridProperties {
                group: Group::default_fill(),
                padding: Default::default(),
                row_spacing: 0,
                col_spacing: 0,
                cells: Vec::new(),
                rows: Vec::new(),
                cols: Vec::new(),
            },
            next_row: 0,
            next_col: 0,
        }
    }

    pub fn with_row_spacing(&mut self, spacing: i32) -> &mut Self {
        self.props.row_spacing = std::cmp::max(0, spacing);
        self
    }

    pub fn with_col_spacing(&mut self, spacing: i32) -> &mut Self {
        self.props.col_spacing = std::cmp::max(0, spacing);
        self
    }

    pub fn with_left_padding(&mut self, padding: i32) -> &mut Self {
        self.props.padding.left = padding;
        self
    }

    pub fn with_top_padding(&mut self, padding: i32) -> &mut Self {
        self.props.padding.top = padding;
        self
    }

    pub fn with_right_padding(&mut self, padding: i32) -> &mut Self {
        self.props.padding.right = padding;
        self
    }

    pub fn with_bottom_padding(&mut self, padding: i32) -> &mut Self {
        self.props.padding.bottom = padding;
        self
    }

    pub fn with_padding(&mut self, left: i32, top: i32, right: i32, bottom: i32) -> &mut Self {
        self.props.padding = Padding {
            left,
            top,
            right,
            bottom,
        };
        self
    }

    pub fn row(&mut self) -> StripeBuilder {
        StripeBuilder::new_row(self)
    }

    pub fn col(&mut self) -> StripeBuilder {
        StripeBuilder::new_col(self)
    }

    pub fn cell(&mut self) -> Option<CellBuilder> {
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

        Some(CellBuilder::new(self, row, col))
    }

    pub fn cell_at(&mut self, row: usize, col: usize) -> Option<CellBuilder> {
        if (row >= self.props.rows.len()) && (col >= self.props.cols.len()) {
            return None;
        }
        match self.props.rows[row].cells[col] {
            StripeCell::Free | StripeCell::Skipped => Some(CellBuilder::new(self, row, col)),
            _ => None,
        }
    }

    pub fn end(self) -> Grid {
        self.props.group.end();
        Grid::new(self.props)
    }
}
