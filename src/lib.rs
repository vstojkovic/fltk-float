use fltk::prelude::WidgetExt;
use fltk::widget::Widget;

pub mod button;
pub mod frame;
pub mod grid;
pub mod input;
pub mod misc;
pub mod text;
mod wrappers;

pub use self::wrappers::{SimpleWrapper, WrapperFactory};

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
