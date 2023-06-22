use super::GridBuilder;
use crate::grid::{Cell, CellAlign, CellProperties, StripeCell};
use crate::{LayoutElement, LayoutWidgetWrapper};

pub struct CellBuilder<'l> {
    owner: &'l mut GridBuilder,
    props: CellProperties,
}

impl<'l> CellBuilder<'l> {
    pub(super) fn new(owner: &'l mut GridBuilder, row: usize, col: usize) -> Self {
        Self {
            owner,
            props: CellProperties {
                row,
                col,
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
        self.owner.props.rows[self.props.row].cells[self.props.col] = StripeCell::Skipped;
        self.owner.props.cols[self.props.col].cells[self.props.row] = StripeCell::Skipped;
    }

    pub fn add<E: LayoutElement + 'static>(self, element: E) {
        let row = self.props.row;
        let col = self.props.col;

        let cell = Cell {
            element: Box::new(element),
            min_size: Default::default(),
            props: self.props,
        };

        let cell_idx = self.owner.props.cells.len();
        self.owner.props.cells.push(cell);

        self.owner.props.rows[row].cells[col] = StripeCell::Cell(cell_idx);
        self.owner.props.cols[col].cells[row] = StripeCell::Cell(cell_idx);
    }

    pub fn wrapped<W: Clone, L: LayoutWidgetWrapper<W> + 'static>(self, wrapper: L) -> W {
        let widget = wrapper.widget();
        self.add(wrapper);
        widget
    }
}
