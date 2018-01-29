use std::default::Default;
use cssparser::{parse_color_keyword, Color};

use super::canvas_element::{CanvasElement};

#[derive(Debug)]
pub struct Context2d<'a> {
  pub canvas: Option<&'a CanvasElement>,
  pub fill_style: Color,
}

impl <'a> Default for Context2d<'a> {
  fn default() -> Context2d<'a> {
    Context2d {
      canvas: None,
      fill_style: parse_color_keyword("black").unwrap(),
    }
  }
}

impl <'a> Context2d<'a> {
  pub fn new(canvas: &'a CanvasElement) -> Context2d {
    Context2d {
      canvas: Some(canvas),
      ..Default::default()
    }
  }
}

#[cfg(test)]
mod context_2d_test {
  use super::{Context2d};
  use cssparser::{Color, RGBA};

  #[test]
  fn context_2d_default_check() {
    let ctx = Context2d { ..Default::default() };
    match ctx.canvas {
      Some(_) => assert!(false),
      None => assert!(true),
    }

    let color = Color::RGBA(RGBA {
      red: 0,
      green: 0,
      blue: 0,
      alpha: 255,
    });
    assert_eq!(ctx.fill_style, color);
  }
}
