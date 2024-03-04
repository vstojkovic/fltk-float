use std::rc::Rc;

use fltk::group::Scroll;
use fltk::prelude::{GroupExt, WidgetBase};

use crate::{LayoutElement, Size};

pub struct Scrollable<G: GroupExt + Clone = Scroll> {
    props: ScrollableProperties<G>,
    child: Rc<dyn LayoutElement>,
}

pub struct ScrollableBuilder<G: GroupExt + Clone = Scroll> {
    props: ScrollableProperties<G>,
}

struct ScrollableProperties<G: GroupExt + Clone> {
    group: G,
    mode: ScrollMode,
    horz_gap: i32,
    vert_gap: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollMode {
    Vertical,
    Horizontal,
    Both,
}

impl<G: GroupExt + Clone> LayoutElement for Scrollable<G> {
    fn min_size(&self) -> Size {
        let scrollbar_size = fltk::app::scrollbar_size();
        let mut min_size = self.child.min_size();
        min_size.width += scrollbar_size + self.props.horz_gap;
        min_size.height += scrollbar_size + self.props.vert_gap;
        if self.props.mode != ScrollMode::Vertical {
            min_size.width = scrollbar_size + self.props.horz_gap;
        }
        if self.props.mode != ScrollMode::Horizontal {
            min_size.height = scrollbar_size + self.props.vert_gap;
        }
        min_size
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.props.group.clone().resize(x, y, width, height);
        self.layout_children();
    }
}

impl Scrollable {
    pub fn builder() -> ScrollableBuilder {
        ScrollableBuilder::new(Scroll::default_fill())
    }
}

impl<G: GroupExt + Clone> Scrollable<G> {
    pub fn group(&self) -> G {
        self.props.group.clone()
    }

    pub fn layout_children(&self) {
        let x = self.props.group.x();
        let y = self.props.group.y();
        let mut width = self.props.group.width();
        let mut height = self.props.group.height();

        let scrollbar_size = fltk::app::scrollbar_size();
        let child_min_size = self.child.min_size();

        let horz_scroll = width < child_min_size.width;
        let vert_scroll = height < child_min_size.height;

        if horz_scroll {
            height -= scrollbar_size + self.props.vert_gap;
        }
        if vert_scroll {
            width -= scrollbar_size + self.props.horz_gap;
        }

        width = std::cmp::max(width, child_min_size.width);
        height = std::cmp::max(height, child_min_size.height);

        self.child.layout(x, y, width, height);
    }

    fn new(props: ScrollableProperties<G>, child: Rc<dyn LayoutElement>) -> Self {
        Self { props, child }
    }
}
impl<G: GroupExt + Clone> ScrollableBuilder<G> {
    pub fn new(group: G) -> Self {
        Self {
            props: ScrollableProperties {
                group,
                mode: ScrollMode::Vertical,
                horz_gap: 0,
                vert_gap: 0,
            },
        }
    }

    pub fn with_mode(mut self, mode: ScrollMode) -> Self {
        self.props.mode = mode;
        self
    }

    pub fn with_horz_gap(mut self, gap: i32) -> Self {
        self.props.horz_gap = gap;
        self
    }

    pub fn with_vert_gap(mut self, gap: i32) -> Self {
        self.props.vert_gap = gap;
        self
    }

    pub fn with_gap(mut self, horz: i32, vert: i32) -> Self {
        self.props.horz_gap = horz;
        self.props.vert_gap = vert;
        self
    }

    pub fn add<E: LayoutElement + 'static>(self, element: E) -> Scrollable<G> {
        self.add_shared(Rc::new(element))
    }

    pub fn add_shared(self, element: Rc<dyn LayoutElement>) -> Scrollable<G> {
        self.props.group.end();
        Scrollable::new(self.props, element)
    }
}
