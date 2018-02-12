extern crate rustcanvas;

use rustcanvas::{create_canvas, CanvasContextType};

fn main() {
  let canvas = create_canvas(1920, 1080);
  canvas.get_context(CanvasContextType::CTX2D).unwrap();
}
