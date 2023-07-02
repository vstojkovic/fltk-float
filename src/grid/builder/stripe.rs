use std::borrow::Borrow;

use fltk::prelude::GroupExt;

use super::GridBuilder;
use crate::grid::{Stripe, StripeCell, StripeProperties};
use crate::WrapperFactory;

pub struct StripeBuilder<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> {
    owner: &'l mut GridBuilder<G, F>,
    props: StripeProperties,
    adder: fn(Self, usize),
}

impl<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> StripeBuilder<'l, G, F> {
    pub(super) fn new_row(owner: &'l mut GridBuilder<G, F>) -> Self {
        Self {
            owner,
            props: StripeProperties { stretch: 0 },
            adder: Self::add_to_rows,
        }
    }

    pub(super) fn new_col(owner: &'l mut GridBuilder<G, F>) -> Self {
        Self {
            owner,
            props: StripeProperties { stretch: 0 },
            adder: Self::add_to_cols,
        }
    }

    pub fn with_stretch(mut self, stretch: u8) -> Self {
        self.props.stretch = stretch;
        self
    }

    pub fn add(self) {
        (self.adder)(self, 1);
    }

    pub fn batch(self, count: usize) {
        (self.adder)(self, count);
    }

    fn add_to_rows(self, count: usize) {
        for _ in 0..count {
            self.owner.props.rows.push(Stripe {
                cells: vec![StripeCell::Free; self.owner.props.cols.len()],
                min_size: 0,
                props: self.props,
            });
            for col in self.owner.props.cols.iter_mut() {
                col.cells.push(StripeCell::Free);
            }
        }
    }

    fn add_to_cols(self, count: usize) {
        for _ in 0..count {
            self.owner.props.cols.push(Stripe {
                cells: vec![StripeCell::Free; self.owner.props.rows.len()],
                min_size: 0,
                props: self.props,
            });
            for row in self.owner.props.rows.iter_mut() {
                row.cells.push(StripeCell::Free);
            }
        }
    }
}
