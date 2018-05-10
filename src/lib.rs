extern crate app_units;
pub extern crate cairo;
extern crate cssparser;
extern crate euclid;
extern crate font_loader as fonts;
#[macro_use]
extern crate lazy_static;
extern crate lyon_path;
extern crate num_traits;
extern crate pathfinder_font_renderer;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod canvas;
mod csshelper;
mod fontrenderer;

pub use canvas::*;

#[cfg(test)]
mod create_canvas_test {
  use canvas::{create_canvas, CanvasContextType};
  #[test]
  fn should_create_canvas() {
    create_canvas(1920, 1080, CanvasContextType::CTX2D);
  }
}
