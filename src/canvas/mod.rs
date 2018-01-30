mod canvas_element;
mod context_2d;
mod paintstate;

use self::canvas_element::{CanvasElement};
pub use self::paintstate::*;

pub fn create_canvas() -> CanvasElement {
  CanvasElement::new()
}
