use super::context_2d::{Context2d};

#[derive(Debug)]
pub struct CanvasElement {
  pixels: Vec<[i8; 4]>,
}

impl <'a> CanvasElement {
  pub fn new() -> CanvasElement {
    CanvasElement { pixels: Vec::new() }
  }

  pub fn get_context(&'a self, context_type: &'a str) -> Option<Context2d> {
    if context_type == "2d" {
      Some(Context2d::new(&self))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod canvas_element_tests {
  use super::CanvasElement;
  #[test]
  fn should_new_canvas() {
    CanvasElement::new();
  }

  #[test]
  fn should_get_context_2d() {
    let element = CanvasElement::new();
    let ctx = element.get_context("2d");
    match ctx {
      Some(_) => assert!(true),
      None => assert!(false),
    };
  }

  #[test]
  fn should_get_none_if_context_type_mismatch() {
    let element = CanvasElement::new();
    let ctx = element.get_context("3d");
    match ctx {
      Some(_) => assert!(false),
      None => assert!(true),
    };
  }
}
