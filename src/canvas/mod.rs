mod canvas_element;
mod context_2d;

use self::canvas_element::{CanvasElement};

pub fn create_canvas() -> CanvasElement {
  CanvasElement::new()
}
