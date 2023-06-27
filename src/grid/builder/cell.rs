use super::GridBuilder;
use crate::grid::{Cell, CellAlign, CellProperties, StripeCell};
use crate::{LayoutElement, LayoutWidgetWrapper};

pub struct CellBuilder<'l> {
    owner: &'l mut GridBuilder,
    props: CellProperties,
}

impl<'l> CellBuilder<'l> {
    pub(super) fn new(
        owner: &'l mut GridBuilder,
        row: usize,
        col: usize,
        row_span: usize,
        col_span: usize,
    ) -> Self {
        Self {
            owner,
            props: CellProperties {
                row,
                col,
                row_span,
                col_span,
                horz_align: CellAlign::Stretch,
                vert_align: CellAlign::Center,
            },
        }
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
        self.owner.add_cell(Cell {
            element: Box::new(element),
            min_size: Default::default(),
            props: self.props,
        });
    }

    pub fn wrapped<W: Clone, L: LayoutWidgetWrapper<W> + 'static>(self, wrapper: L) -> W {
        let widget = wrapper.widget();
        self.add(wrapper);
        widget
    }
}
