use std::ops::{Deref, DerefMut};

use fltk::prelude::*;

use super::LayoutElement;

pub struct InputElement<I: InputExt + Clone> {
    widget: I,
}

impl<I: InputExt + Clone> InputElement<I> {
    pub fn wrap(widget: I) -> Self {
        Self { widget }
    }
}

impl<I: InputExt + Clone> LayoutElement for InputElement<I> {
    fn min_size(&self) -> (i32, i32) {
        fltk::draw::set_font(self.widget.text_font(), self.widget.text_size());
        let text_height = fltk::draw::height();
        let frame = self.widget.frame();
        let frame_dx = frame.dx();
        let frame_dy = frame.dy();
        let frame_dw = frame.dw();
        let frame_dh = frame.dh();
        let frame_width = frame_dx + frame_dw;
        let frame_height = frame_dy + frame_dh;
        (frame_width, text_height + frame_height)
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.widget.clone().resize(x, y, width, height)
    }
}

impl<I: InputExt + Clone> Deref for InputElement<I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<I: InputExt + Clone> DerefMut for InputElement<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
