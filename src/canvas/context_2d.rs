use super::canvas_element::{CanvasElement};
use super::paintstate::{PaintState};

#[derive(Debug)]
pub struct Context2d<'a> {
  pub canvas: Option<&'a CanvasElement>,
  pub state: PaintState,
}

impl <'a> Context2d<'a> {
  pub fn new(canvas: &'a CanvasElement) -> Context2d {
    Context2d {
      canvas: Some(canvas),
      state: PaintState::new(),
    }
  }
}

#[cfg(test)]
mod context_2d_test {
  use super::{Context2d};
  use super::super::{create_canvas};

  #[test]
  fn new_context_2d_check() {
    let canvas = & create_canvas();
    Context2d::new(canvas);
  }
}
