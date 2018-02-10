mod fillstyle;
mod font;

use std::default::Default;
use cssparser::{parse_color_keyword};
pub use self::fillstyle::*;
pub use self::font::*;

#[derive(Debug)]
pub struct PaintState {
  pub fill_style: FillStyle,
  pub font: Font,
}

impl Default for PaintState {
  fn default() -> Self {
    PaintState {
      fill_style: FillStyle::Color(parse_color_keyword("black").unwrap()),
      font: Font::new("10px sans-serif"),
    }
  }
}

impl PaintState {
  pub fn new() -> PaintState {
    PaintState::default()
  }
}

#[cfg(test)]
mod paint_state_test {
  use super::{PaintState, FillStyle};
  use cssparser::{RGBA, Color};

  #[test]
  fn paint_state_default_check() {
    let state = PaintState::default();
    match state.fill_style {
      FillStyle::Color(c) => {
        assert_eq!(c, Color::RGBA(RGBA::new(0, 0, 0, 255)));
      },
      _ => assert!(false),
    };
  }
}
