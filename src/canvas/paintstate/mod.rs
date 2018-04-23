mod font;

pub use self::font::*;
use super::canvas_trait::CairoPattern;
use cssparser::RGBA;
use euclid::Transform2D;
use std::default::Default;

#[derive(Debug, Clone)]
pub struct PaintState {
  pub fill_style: CairoPattern,
  pub stroke_style: CairoPattern,
  pub font: Font,
  pub global_alpha: f64,
  pub transform: Transform2D<f64>,
  pub shadow_offset_x: f64,
  pub shadow_offset_y: f64,
  pub shadow_blur: f64,
  pub shadow_color: RGBA,
}

impl Default for PaintState {
  fn default() -> Self {
    let fill_style = CairoPattern::Color(RGBA::new(0, 0, 0, 1));
    let stroke_style = fill_style.clone();
    PaintState {
      fill_style,
      stroke_style,
      font: Font::new("10px sans-serif"),
      transform: Transform2D::identity(),
      global_alpha: 1.0f64,
      shadow_offset_x: 0.0,
      shadow_offset_y: 0.0,
      shadow_blur: 0.0,
      shadow_color: RGBA {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
      },
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
  use super::*;
  use cssparser::RGBA;
  use std::mem;

  struct TestColorPattern {
    pub color: Color,
    pub azure_color_pattern: AzColorPatternRef,
  }

  #[test]
  fn paint_state_default_check() {
    let state = PaintState::default();
    match state.fill_style {
      Pattern::Color(c) => {
        let color: Color = unsafe {
          let color_pattern: TestColorPattern = mem::transmute(c);
          color_pattern.color
        };
        assert_eq!(color, RGBA::new(0, 0, 0, 1).to_azure_style());
      }
      _ => assert!(false),
    };
    match state.stroke_style {
      Pattern::Color(c) => {
        let color: Color = unsafe {
          let color_pattern: TestColorPattern = mem::transmute(c);
          color_pattern.color
        };
        assert_eq!(color, RGBA::new(0, 0, 0, 1).to_azure_style());
      }
      _ => assert!(false),
    };
  }
}
