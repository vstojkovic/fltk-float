use std::borrow::Borrow;
use std::rc::Rc;

use fltk::prelude::GroupExt;

use crate::grid::{Cell, CellAlign, CellProperties, Padding, StripeCell};
use crate::{IntoWidget, LayoutElement, WrapperFactory};

use super::GridBuilder;

pub struct CellBuilder<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> {
    owner: &'l mut GridBuilder<G, F>,
    props: CellProperties,
}

impl<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> CellBuilder<'l, G, F> {
    pub(super) fn new(
        owner: &'l mut GridBuilder<G, F>,
        row: usize,
        col: usize,
        row_span: usize,
        col_span: usize,
    ) -> Self {
        let padding = owner.default_cell_padding;
        let horz_align = owner.default_col_align[col];
        let vert_align = owner.default_row_align[row];
        Self {
            owner,
            props: CellProperties {
                row,
                col,
                row_span,
                col_span,
                padding,
                horz_align,
                vert_align,
            },
        }
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

    pub fn with_horz_align(mut self, align: CellAlign) -> Self {
        self.props.horz_align = align;
        self
    }

    pub fn with_vert_align(mut self, align: CellAlign) -> Self {
        self.props.vert_align = align;
        self
    }

    pub fn skip(self) {
        let top = self.props.row;
        let bottom = top + self.props.row_span;
        let left = self.props.col;
        let right = left + self.props.col_span;
        for row in top..bottom {
            for col in left..right {
                self.owner.props.rows[row].cells[col] = StripeCell::Skipped;
                self.owner.props.cols[col].cells[row] = StripeCell::Skipped;
            }
        }
    }

    pub fn add<E: LayoutElement + 'static>(self, element: E) {
        self.add_shared(Rc::new(element));
    }

    pub fn add_shared(self, element: Rc<dyn LayoutElement>) {
        self.owner.add_cell(Cell {
            element,
            min_size: Default::default(),
            props: self.props,
        });
    }

    pub fn wrap<W: IntoWidget + 'static>(self, widget: W) -> W {
        let element = self.owner.factory.borrow().wrap(widget.clone());
        self.add_shared(element);
        widget
    }
}
