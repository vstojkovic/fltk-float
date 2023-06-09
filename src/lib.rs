pub mod button;
pub mod frame;
pub mod input;
pub mod misc;

pub trait LayoutElement {
    fn min_size(&self) -> (i32, i32);
    fn layout(&self, x: i32, y: i32, width: i32, height: i32);
}
