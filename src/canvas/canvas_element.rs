use euclid::{Rect, Size2D};

use super::context_2d::{Context2d};

pub struct CanvasElement {
  pub width: i32,
  pub height: i32,
  pub ctx: Context2d<'static>,
}

#[derive(Debug)]
pub enum CanvasContextType {
  CTX2D,
  WEBGL,
  WEBGL2,
  BITMAPRENDERER,
}

impl <'a> CanvasElement {
  pub fn new(width: i32, height: i32, context_type: CanvasContextType) -> Option<CanvasElement> {
    match context_type {
      CanvasContextType::CTX2D => {
        let ctx = Context2d::new(width, height);
        Some(CanvasElement { width, height, ctx })
      },
      _ => None,
    }
  }

  pub fn image_data(&self, dest_rect: Rect<i32>, canvas_size: Size2D<f64>) -> Vec<u8> {
    self.ctx.image_data(dest_rect, canvas_size)
  }
}

#[cfg(test)]
mod canvas_element_tests {
  use super::*;
  #[test]
  fn should_new_canvas() {
    CanvasElement::new(1920, 1080, CanvasContextType::CTX2D);
  }

  #[test]
  fn should_get_context_2d() {
    let element = CanvasElement::new(1920, 1080, CanvasContextType::CTX2D);
    match element {
      Some(_) => assert!(true),
      None => assert!(false),
    };
  }

  #[test]
  fn should_get_none_if_context_type_mismatch() {
    let element = CanvasElement::new(1920, 1080, CanvasContextType::WEBGL2);
    match element {
      Some(_) => assert!(false),
      None => assert!(true),
    };
  }
}
