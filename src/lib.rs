extern crate cssparser;
extern crate font_rs;

mod canvas;
mod csshelper;

pub use canvas::{create_canvas};

#[cfg(test)]
mod create_canvas_test {
  use canvas::{create_canvas};
  #[test]
  fn should_create_canvas() {
    create_canvas();
  }
}
