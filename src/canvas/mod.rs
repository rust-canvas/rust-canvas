mod canvas_element;
mod canvas_trait;
mod context_2d;
mod paintstate;

pub use self::canvas_element::{CanvasElement, CanvasContextType};
pub use self::paintstate::*;
pub use self::canvas_trait::*;
pub use self::context_2d::*;

pub fn create_canvas(width: i32, height: i32, ctx_type: CanvasContextType) -> CanvasElement {
  CanvasElement::new(width, height, ctx_type).unwrap()
}
