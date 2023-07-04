use std::borrow::Borrow;

use fltk::prelude::GroupExt;

use crate::grid::StripeProperties;
use crate::WrapperFactory;

use super::{GridBuilder, StripeGroupRef, StripeKind};

pub struct StripeGroupBuilder<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> {
    owner: &'l mut GridBuilder<G, F>,
    kind: StripeKind,
    props: StripeProperties,
}

impl<'l, G: GroupExt + Clone, F: Borrow<WrapperFactory>> StripeGroupBuilder<'l, G, F> {
    pub(super) fn new(owner: &'l mut GridBuilder<G, F>, kind: StripeKind) -> Self {
        Self {
            owner,
            kind,
            props: StripeProperties {
                stretch: 0,
                min_size: 0,
            },
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

    pub fn add(self) -> StripeGroupRef {
        let idx = self.owner.props.groups.len();
        self.owner.props.groups.push(self.props);
        StripeGroupRef {
            kind: self.kind,
            idx,
        }
    }
}
