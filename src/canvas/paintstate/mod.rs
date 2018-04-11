mod font;

use std::default::Default;
use azure::azure_hl::{Color, StrokeOptions, JoinStyle, CapStyle, DrawOptions};
use azure::azure_hl::{AntialiasMode, CompositionOp};
use azure::{AzFloat};
use euclid::{Transform2D};
use cssparser::{RGBA};
use super::context_2d::{ToAzureStyle};
use super::canvas_trait::{CairoPattern};
pub use self::font::*;

#[derive(Debug, Clone)]
pub struct PaintState<'a> {
  pub draw_options: DrawOptions,
  pub fill_style: CairoPattern,
  pub stroke_style: CairoPattern,
  pub stroke_opts: StrokeOptions<'a>,
  pub font: Font,
  pub transform: Transform2D<f64>,
  pub shadow_offset_x: f64,
  pub shadow_offset_y: f64,
  pub shadow_blur: f64,
  pub shadow_color: Color,
}

impl <'a> Default for PaintState<'a> {
  fn default() -> Self {
    let fill_style = CairoPattern::Color(RGBA::new(0, 0, 0, 1));
    let stroke_style = fill_style.clone();
    PaintState {
      draw_options: DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::Default),
      fill_style,
      stroke_style,
      stroke_opts: StrokeOptions::new(1.0, JoinStyle::MiterOrBevel, CapStyle::Butt, 10.0, &[]),
      font: Font::new("10px sans-serif"),
      transform: Transform2D::identity(),
      shadow_offset_x: 0.0,
      shadow_offset_y: 0.0,
      shadow_blur: 0.0,
      shadow_color: Color::transparent(),
    }
  }
}

impl <'a> PaintState <'a> {
  pub fn new() -> PaintState<'a> {
    PaintState::default()
  }
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
    match state.stroke_style {
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
