use super::context_2d::{Context2d};

#[derive(Debug)]
pub struct CanvasElement {
  pub width: i32,
  pub height: i32,
}

#[derive(Debug)]
pub enum CanvasContextType {
  CTX2D,
  WEBGL,
  WEBGL2,
  BITMAPRENDERER,
}

impl <'a> CanvasElement {
  pub fn new(width: i32, height: i32) -> CanvasElement {
    CanvasElement { width, height }
  }

  pub fn get_context(&'a self, context_type: CanvasContextType) -> Option<Context2d> {
    match context_type {
      CanvasContextType::CTX2D => Some(Context2d::new(&self)),
      _ => None,
    }
  }
}

#[cfg(test)]
mod canvas_element_tests {
  use super::*;
  #[test]
  fn should_new_canvas() {
    CanvasElement::new(1920, 1080);
  }

  #[test]
  fn should_get_context_2d() {
    let element = CanvasElement::new(1920, 1080);
    let ctx = element.get_context(CanvasContextType::CTX2D);
    match ctx {
      Some(_) => assert!(true),
      None => assert!(false),
    };
  }

  #[test]
  fn should_get_none_if_context_type_mismatch() {
    let element = CanvasElement::new(1920, 1080);
    let ctx = element.get_context(CanvasContextType::WEBGL2);
    match ctx {
      Some(_) => assert!(false),
      None => assert!(true),
    };
  }
}
