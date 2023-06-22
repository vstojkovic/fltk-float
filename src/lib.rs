pub mod button;
pub mod frame;
pub mod input;
pub mod misc;

pub trait LayoutElement {
    fn min_size(&self) -> Size;
    fn layout(&self, x: i32, y: i32, width: i32, height: i32);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}
