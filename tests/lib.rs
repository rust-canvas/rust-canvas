extern crate rustcanvas;

#[cfg(test)]
mod intergration_tests {
  use rustcanvas::{create_canvas, CanvasContextType};

  #[test]
  fn should_create_canvas() {
    create_canvas(1920, 1080, CanvasContextType::CTX2D);
  }
}
