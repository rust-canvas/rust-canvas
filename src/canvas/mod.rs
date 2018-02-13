mod canvas_element;
mod canvas_trait;
mod context_2d;
mod paintstate;

pub use self::canvas_element::{CanvasElement, CanvasContextType};
pub use self::paintstate::*;
pub use self::canvas_trait::*;

pub fn create_canvas(width: i32, height: i32) -> CanvasElement {
  CanvasElement::new(width, height)
}
