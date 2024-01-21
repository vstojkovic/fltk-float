use std::borrow::Borrow;
use std::rc::Rc;

use fltk::group::Group;
use fltk::prelude::{GroupExt, WidgetBase};

use crate::{IntoWidget, LayoutElement, Padding, Size, WrapperFactory};

pub struct Overlay<G: GroupExt + Clone = Group> {
    props: OverlayProperties<G>,
    min_size: Size,
}

pub struct OverlayBuilder<G: GroupExt + Clone = Group, F: Borrow<WrapperFactory> = WrapperFactory> {
    props: OverlayProperties<G>,
    factory: F,
}

struct OverlayProperties<G: GroupExt + Clone> {
    group: G,
    padding: Padding,
    children: Vec<Rc<dyn LayoutElement>>,
}

impl<G: GroupExt + Clone> LayoutElement for Overlay<G> {
    fn min_size(&self) -> Size {
        self.min_size
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.props.group.clone().resize(x, y, width, height);
        self.layout_children();
    }
}

impl Overlay {
    pub fn builder() -> OverlayBuilder<Group, WrapperFactory> {
        OverlayBuilder::new(Group::default_fill())
    }

    pub fn builder_with_factory<F: Borrow<WrapperFactory>>(factory: F) -> OverlayBuilder<Group, F> {
        OverlayBuilder::with_factory(Group::default_fill(), factory)
    }
}

impl<G: GroupExt + Clone> Overlay<G> {
    pub fn group(&self) -> G {
        self.props.group.clone()
    }

    pub fn layout_children(&self) {
        let x = self.props.group.x() + self.props.padding.left;
        let y = self.props.group.y() + self.props.padding.top;
        let width = self.props.group.width() - (self.props.padding.left + self.props.padding.right);
        let height =
            self.props.group.height() - (self.props.padding.top + self.props.padding.bottom);

        for child in self.props.children.iter() {
            child.layout(x, y, width, height);
        }
    }

    fn new(props: OverlayProperties<G>) -> Self {
        let mut min_size = props.children.iter().map(|child| child.min_size()).fold(
            Default::default(),
            |lhs: Size, rhs: Size| Size {
                width: std::cmp::max(lhs.width, rhs.width),
                height: std::cmp::max(lhs.height, rhs.height),
            },
        );
        min_size.width += props.padding.left + props.padding.right;
        min_size.height += props.padding.top + props.padding.bottom;

        Self { props, min_size }
    }
}

impl<G: GroupExt + Clone> OverlayBuilder<G> {
    pub fn new(group: G) -> Self {
        Self::with_factory(group, WrapperFactory::new())
    }
}

impl<G: GroupExt + Clone, F: Borrow<WrapperFactory>> OverlayBuilder<G, F> {
    pub fn with_factory(group: G, factory: F) -> Self {
        Self {
            props: OverlayProperties {
                group,
                padding: Default::default(),
                children: Vec::new(),
            },
            factory,
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

    pub fn add<E: LayoutElement + 'static>(&mut self, element: E) {
        self.add_shared(Rc::new(element));
    }

    pub fn add_shared(&mut self, element: Rc<dyn LayoutElement>) {
        self.props.children.push(element);
    }

    pub fn wrap<W: IntoWidget + 'static>(&mut self, widget: W) -> W {
        let element = self.factory.borrow().wrap(widget.clone());
        self.add_shared(element);
        widget
    }

    pub fn end(self) -> Overlay<G> {
        self.props.group.end();
        Overlay::new(self.props)
    }
}
