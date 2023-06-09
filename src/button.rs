use std::cmp::max;
use std::ops::{Deref, DerefMut};

use fltk::prelude::*;

use super::LayoutElement;

pub struct ButtonElement<B: ButtonExt + Clone> {
    widget: B,
}

impl<B: ButtonExt + Clone> ButtonElement<B> {
    pub fn wrap(widget: B) -> Self {
        Self { widget }
    }
}

impl<B: ButtonExt + Clone> LayoutElement for ButtonElement<B> {
    fn min_size(&self) -> (i32, i32) {
        let (label_width, label_height) = self.widget.measure_label();
        let up_frame = self.widget.frame();
        let down_frame = self.widget.down_frame();
        let frame_dx = max(up_frame.dx(), down_frame.dx());
        let frame_dy = max(up_frame.dy(), down_frame.dy());
        let frame_dw = max(up_frame.dw(), down_frame.dw());
        let frame_dh = max(up_frame.dh(), down_frame.dh());
        let frame_width = frame_dx + frame_dw;
        let frame_height = frame_dy + frame_dh;

        (
            label_width + frame_width + label_height,
            label_height + 2 * frame_height,
        )
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.widget.clone().resize(x, y, width, height)
    }
}

impl<B: ButtonExt + Clone> Deref for ButtonElement<B> {
    type Target = B;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<B: ButtonExt + Clone> DerefMut for ButtonElement<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
