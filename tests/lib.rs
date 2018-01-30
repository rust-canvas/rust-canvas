extern crate rustcanvas;

#[cfg(test)]
mod intergration_tests {
  use rustcanvas::{create_canvas};

  #[test]
  fn should_create_canvas() {
    create_canvas();
  }
}
