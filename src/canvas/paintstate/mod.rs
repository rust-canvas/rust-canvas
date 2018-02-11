mod font;

use std::default::Default;
use azure::azure_hl::{Pattern, ColorPattern, Color};
use azure::{AzFloat};
use cssparser::{RGBA};
pub use self::font::*;

#[derive(Debug)]
pub struct PaintState {
  pub fill_style: Pattern,
  pub font: Font,
}

impl Default for PaintState {
  fn default() -> Self {
    PaintState {
      fill_style: Pattern::Color(ColorPattern::new(RGBA::new(0, 0, 0, 1).to_azure_style())),
      font: Font::new("10px sans-serif"),
    }
  }
}

impl PaintState {
  pub fn new() -> PaintState {
    PaintState::default()
  }
}

pub trait ToAzureStyle {
    type Target;
    fn to_azure_style(self) -> Self::Target;
}

impl ToAzureStyle for RGBA {
  type Target = Color;

  fn to_azure_style(self) -> Color {
    Color::rgba(self.red_f32() as AzFloat,
                self.green_f32() as AzFloat,
                self.blue_f32() as AzFloat,
                self.alpha_f32() as AzFloat)
  }
}

#[cfg(test)]
mod paint_state_test {
  use super::*;
  use std::mem;
  use cssparser::{RGBA};
  use azure::azure_hl::{Pattern, Color};
  use azure::{AzColorPatternRef};

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
      },
      _ => assert!(false),
    };
  }
}
