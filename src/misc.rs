use std::ops::{Deref, DerefMut};

use fltk::misc::InputChoice;
use fltk::prelude::*;

use super::{LayoutElement, Size};

pub struct InputChoiceElement {
    widget: InputChoice,
}

impl InputChoiceElement {
    pub fn wrap(widget: InputChoice) -> Self {
        Self { widget }
    }
}

impl LayoutElement for InputChoiceElement {
    fn min_size(&self) -> Size {
        fltk::draw::set_font(self.widget.text_font(), self.widget.text_size());
        let text_height = fltk::draw::height();
        let text_width = self
            .widget
            .menu_button()
            .into_iter()
            .filter_map(|item| item.label())
            .map(|label| fltk::draw::measure(&label, true).0)
            .max()
            .unwrap_or_default();

        let frame = self.widget.frame();
        let frame_dx = frame.dx();
        let frame_dy = frame.dy();
        let frame_dw = frame.dw();
        let frame_dh = frame.dh();
        let frame_width = frame_dx + frame_dw;
        let frame_height = frame_dy + frame_dh;

        Size {
            width: 3 * frame_width + text_width + text_height,
            height: text_height + frame_height,
        }
    }

    fn layout(&self, x: i32, y: i32, width: i32, height: i32) {
        self.widget.clone().resize(x, y, width, height);
    }
}

impl Deref for InputChoiceElement {
    type Target = InputChoice;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl DerefMut for InputChoiceElement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}
