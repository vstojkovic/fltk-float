use std::ops::{Deref, DerefMut};

use fltk::prelude::*;

use crate::{LayoutElement, LayoutWidgetWrapper, Size};

pub struct TextElement<T: DisplayExt + Clone> {
    widget: T,
}

impl<T: DisplayExt + Clone> LayoutWidgetWrapper<T> for TextElement<T> {
    fn wrap(widget: T) -> Self {
        Self { widget }
    }
}

impl<T: DisplayExt + Clone> LayoutElement for TextElement<T> {
    fn min_size(&self) -> crate::Size {
        fltk::draw::set_font(self.widget.text_font(), self.widget.text_size());
        let text_height = fltk::draw::height();
        let frame = self.widget.frame();
        let frame_dx = frame.dx();
        let frame_dy = frame.dy();
        let frame_dw = frame.dw();
        let frame_dh = frame.dh();
        let frame_width = frame_dx + frame_dw;
        let frame_height = frame_dy + frame_dh;
        Size {
            width: frame_width,
            height: text_height + frame_height + 1,
        }
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.widget.clone().resize(x, y, width, height);
    }
}

impl<T: DisplayExt + Clone> Deref for TextElement<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<T: DisplayExt + Clone> DerefMut for TextElement<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
