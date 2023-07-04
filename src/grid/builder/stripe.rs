use std::borrow::Borrow;

use fltk::prelude::GroupExt;

use super::{GridBuilder, StripeKind};
use crate::grid::{CellAlign, Stripe, StripeCell, StripeProperties};
use crate::WrapperFactory;

pub struct StripeBuilder<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> {
    owner: &'l mut GridBuilder<G, F>,
    kind: StripeKind,
    props: StripeProperties,
    group_idx: Option<usize>,
    default_align: CellAlign,
}

impl<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> StripeBuilder<'l, G, F> {
    pub(super) fn new(
        owner: &'l mut GridBuilder<G, F>,
        kind: StripeKind,
        group_idx: Option<usize>,
    ) -> Self {
        let default_align = match kind {
            StripeKind::Row => CellAlign::Center,
            StripeKind::Column => CellAlign::Stretch,
        };
        Self {
            owner,
            kind,
            props: StripeProperties {
                stretch: 0,
                min_size: 0,
            },
            group_idx,
            default_align,
        }
    }

    pub fn with_stretch(mut self, stretch: u8) -> Self {
        self.props.stretch = stretch;
        self
    }

    pub fn with_min_size(mut self, min_size: i32) -> Self {
        self.props.min_size = std::cmp::max(0, min_size);
        self
    }

    pub fn with_default_align(mut self, align: CellAlign) -> Self {
        self.default_align = align;
        self
    }

    pub fn add(self) {
        self.add_to_owner(1);
    }

    pub fn batch(self, count: usize) {
        self.add_to_owner(count);
    }

    fn add_to_owner(self, count: usize) {
        let (stripes, default_aligns, perpendicular) = match self.kind {
            StripeKind::Row => (
                &mut self.owner.props.rows,
                &mut self.owner.default_row_align,
                &mut self.owner.props.cols,
            ),
            StripeKind::Column => (
                &mut self.owner.props.cols,
                &mut self.owner.default_col_align,
                &mut self.owner.props.rows,
            ),
        };
        for _ in 0..count {
            let group_idx = self.group_idx.unwrap_or_else(|| {
                let idx = self.owner.props.groups.len();
                self.owner.props.groups.push(self.props);
                idx
            });
            stripes.push(Stripe {
                cells: vec![StripeCell::Free; perpendicular.len()],
                group_idx,
            });
            default_aligns.push(self.default_align);
            for perp in perpendicular.iter_mut() {
                perp.cells.push(StripeCell::Free);
            }
        }
    }
}
