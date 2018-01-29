extern crate cssparser;

pub mod canvas;

#[cfg(test)]
mod create_canvas_test {
  use canvas::{create_canvas};
  #[test]
  fn should_create_canvas() {
    create_canvas();
  }
}
