use std::any::{Any, TypeId};
use std::collections::HashMap;

use fltk::prelude::WidgetExt;
use fltk::widget::Widget;

pub mod button;
pub mod frame;
pub mod grid;
pub mod input;
pub mod misc;
pub mod text;

pub trait LayoutElement {
    fn min_size(&self) -> Size;
    fn layout(&self, x: i32, y: i32, width: i32, height: i32);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

pub trait IntoWidget: Clone {
    fn into_widget(self) -> Widget;
}

impl<W: WidgetExt + Clone> IntoWidget for W {
    fn into_widget(self) -> Widget {
        self.as_base_widget()
    }
}

pub trait LayoutWidgetWrapper<W: IntoWidget>: LayoutElement {
    fn wrap(widget: W) -> Self;
}

pub struct SimpleWrapper {
    pub widget: Widget,
    pub min_size: Size,
}

impl LayoutElement for SimpleWrapper {
    fn min_size(&self) -> Size {
        self.min_size
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.widget.clone().resize(x, y, width, height);
    }
}

impl<W: IntoWidget> LayoutWidgetWrapper<W> for SimpleWrapper {
    fn wrap(widget: W) -> Self {
        let widget = widget.into_widget();
        let min_size = Size {
            width: widget.width(),
            height: widget.height(),
        };
        Self::new(widget, min_size)
    }
}

impl SimpleWrapper {
    pub fn new<W: IntoWidget>(widget: W, min_size: Size) -> Self {
        Self {
            widget: widget.into_widget(),
            min_size,
        }
    }
}

pub struct WrapperFactory {
    map: HashMap<TypeId, Box<dyn Any>>,
    catch_all: Box<dyn Fn(Widget) -> Box<dyn LayoutElement>>,
}

impl WrapperFactory {
    pub fn new() -> Self {
        Self::with_catch_all(|widget| Box::new(SimpleWrapper::wrap(widget)))
    }

    pub fn with_catch_all(catch_all: impl Fn(Widget) -> Box<dyn LayoutElement> + 'static) -> Self {
        Self {
            map: HashMap::new(),
            catch_all: Box::new(catch_all),
        }
    }

    pub fn set_wrapper<W: IntoWidget + 'static, L: LayoutWidgetWrapper<W> + 'static>(&mut self) {
        self.map
            .insert(TypeId::of::<W>(), Box::new(Factory::<W>::new::<L>()));
    }

    pub fn wrap<W: IntoWidget + 'static>(&self, widget: W) -> Box<dyn LayoutElement> {
        match self.factory_for::<W>() {
            Some(factory) => (factory.0)(widget),
            None => (self.catch_all)(widget.into_widget()),
        }
    }

    fn factory_for<W: IntoWidget + 'static>(&self) -> Option<&Factory<W>> {
        let erased = self.map.get(&TypeId::of::<W>())?;
        let erased = &*erased;
        erased.downcast_ref::<Factory<W>>()
    }
}

struct Factory<W: IntoWidget + 'static>(fn(W) -> Box<dyn LayoutElement>);

impl<W: IntoWidget + 'static> Factory<W> {
    fn new<L: LayoutWidgetWrapper<W> + 'static>() -> Self {
        Self(|widget| Box::new(L::wrap(widget)))
    }
}
