use std::ops::{Deref, DerefMut};

use fltk::frame::Frame;
use fltk::prelude::*;

use super::{LayoutElement, LayoutWidgetWrapper, Size};

pub struct FrameElement {
    widget: Frame,
}

impl LayoutWidgetWrapper<Frame> for FrameElement {
    fn wrap(widget: Frame) -> Self {
        Self { widget }
    }
    fn widget(&self) -> Frame {
        self.widget.clone()
    }
}

impl LayoutElement for FrameElement {
    fn min_size(&self) -> Size {
        let (label_width, label_height) = self.widget.measure_label();
        let frame = self.widget.frame();
        let frame_dx = frame.dx();
        let frame_dy = frame.dy();
        let frame_dw = frame.dw();
        let frame_dh = frame.dh();
        let frame_width = frame_dx + frame_dw;
        let frame_height = frame_dy + frame_dh;
        Size {
            width: label_width + 2 * frame_width,
            height: label_height + frame_height,
        }
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.widget().resize(x, y, width, height);
    }
}

impl Deref for FrameElement {
    type Target = Frame;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl DerefMut for FrameElement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
