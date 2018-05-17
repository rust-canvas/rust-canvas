use std::fmt::{Debug, Error, Formatter};
use std::str::FromStr;
use std::sync::mpsc::Sender;

use cairo::{LinearGradient, PatternTrait, RadialGradient, SolidPattern, SurfacePattern};
use cssparser::RGBA;
use euclid::{Point2D, Rect, Size2D, Transform2D, Vector2D};

pub enum CairoPattern {
  Color(RGBA),
  SolidPattern(SolidPattern),
  LinearGradient(LinearGradient),
  RadialGradient(RadialGradient),
  SurfacePattern(SurfacePattern),
}

impl Clone for CairoPattern {
  fn clone(&self) -> CairoPattern {
    match *self {
      CairoPattern::Color(ref c) => CairoPattern::Color(c.clone()),
      CairoPattern::SolidPattern(ref s) => CairoPattern::SolidPattern(s.reference()),
      CairoPattern::LinearGradient(ref l) => CairoPattern::LinearGradient(l.reference()),
      CairoPattern::RadialGradient(ref r) => CairoPattern::RadialGradient(r.reference()),
      CairoPattern::SurfacePattern(ref s) => CairoPattern::SurfacePattern(s.reference()),
    }
  }
}

impl Debug for CairoPattern {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    match *self {
      CairoPattern::Color(ref c) => write!(f, "CairoPattern::Color({:?})", c,),
      CairoPattern::LinearGradient(ref linear) => write!(
        f,
        "CairoPattern::LinearGradient{:?}",
        linear.get_linear_points()
      ),
      CairoPattern::RadialGradient(ref radial) => write!(
        f,
        "CairoPattern::RadialGradient{:?}",
        radial.get_radial_circles()
      ),
      CairoPattern::SolidPattern(ref solid) => {
        write!(f, "CairoPattern::SolidPattern(RGBA{:?})", solid.get_rgba())
      }
      CairoPattern::SurfacePattern(ref surface) => write!(
        f,
        "CairoPattern::SurfacePattern({:?})",
        surface.get_surface()
      ),
    }
  }
}

#[derive(Clone, Debug)]
pub enum CanvasMsg {
  Canvas2d(Canvas2dMsg),
  GetImageData(Rect<i32>, Size2D<f64>, Sender<Vec<u8>>),
  FromScript(FromScriptMsg),
  Close,
}

#[derive(Clone, Debug)]
pub enum FromScriptMsg {
  SendPixels(Sender<Option<Vec<u8>>>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Canvas2dMsg {
  Arc(Point2D<f64>, f64, f64, f64, bool),
  ArcTo(Point2D<f64>, Point2D<f64>, f64),
  DrawImage(Vec<u8>, Size2D<f64>, Rect<f64>, Rect<f64>, bool),
  DrawImageURI(String, Size2D<f64>, Rect<f64>, Rect<f64>, bool),
  DrawImageSelf(Size2D<f64>, Rect<f64>, Rect<f64>, bool),
  BeginPath,
  BezierCurveTo(Point2D<f64>, Point2D<f64>, Point2D<f64>),
  ClearRect(Rect<f64>),
  Clip,
  ClosePath,
  Ellipse(Point2D<f64>, f64, f64, f64, f64, f64, bool),
  Fill,
  FillText(String, f32, f32, Option<f32>),
  FillRect(Rect<f64>),
  LineTo(Point2D<f64>),
  MoveTo(Point2D<f64>),
  PutImageData(Vec<u8>, Vector2D<f64>, Size2D<f64>, Rect<f64>),
  QuadraticCurveTo(Point2D<f64>, Point2D<f64>),
  Rect(Rect<f64>),
  RestoreContext,
  SaveContext,
  StrokeRect(Rect<f64>),
  Stroke,
  StrokeText(String, f32, f32, Option<f32>),
  SetFillStyle(FillOrStrokeStyle),
  SetFontStyle(String),
  SetStrokeStyle(FillOrStrokeStyle),
  SetLineWidth(f64),
  SetLineCap(LineCapStyle),
  SetLineJoin(LineJoinStyle),
  SetMiterLimit(f64),
  SetGlobalAlpha(f64),
  SetGlobalComposition(CompositionOrBlending),
  SetTransform(Transform2D<f64>),
  SetShadowOffsetX(f64),
  SetShadowOffsetY(f64),
  SetShadowBlur(f64),
  SetShadowColor(RGBA),
  // for not implement methods
  NotImplement,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum FillRule {
  Nonzero,
  Evenodd,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize, Debug)]
pub enum BlendingStyle {
  Multiply,
  Screen,
  Overlay,
  Darken,
  Lighten,
  ColorDodge,
  ColorBurn,
  HardLight,
  SoftLight,
  Difference,
  Exclusion,
  Hue,
  Saturation,
  Color,
  Luminosity,
}

impl FromStr for BlendingStyle {
  type Err = ();

  fn from_str(string: &str) -> Result<BlendingStyle, ()> {
    match string {
      "multiply" => Ok(BlendingStyle::Multiply),
      "screen" => Ok(BlendingStyle::Screen),
      "overlay" => Ok(BlendingStyle::Overlay),
      "darken" => Ok(BlendingStyle::Darken),
      "lighten" => Ok(BlendingStyle::Lighten),
      "color-dodge" => Ok(BlendingStyle::ColorDodge),
      "color-burn" => Ok(BlendingStyle::ColorBurn),
      "hard-light" => Ok(BlendingStyle::HardLight),
      "soft-light" => Ok(BlendingStyle::SoftLight),
      "difference" => Ok(BlendingStyle::Difference),
      "exclusion" => Ok(BlendingStyle::Exclusion),
      "hue" => Ok(BlendingStyle::Hue),
      "saturation" => Ok(BlendingStyle::Saturation),
      "color" => Ok(BlendingStyle::Color),
      "luminosity" => Ok(BlendingStyle::Luminosity),
      _ => Err(()),
    }
  }
}

impl BlendingStyle {
  pub fn to_str(&self) -> &str {
    match *self {
      BlendingStyle::Multiply => "multiply",
      BlendingStyle::Screen => "screen",
      BlendingStyle::Overlay => "overlay",
      BlendingStyle::Darken => "darken",
      BlendingStyle::Lighten => "lighten",
      BlendingStyle::ColorDodge => "color-dodge",
      BlendingStyle::ColorBurn => "color-burn",
      BlendingStyle::HardLight => "hard-light",
      BlendingStyle::SoftLight => "soft-light",
      BlendingStyle::Difference => "difference",
      BlendingStyle::Exclusion => "exclusion",
      BlendingStyle::Hue => "hue",
      BlendingStyle::Saturation => "saturation",
      BlendingStyle::Color => "color",
      BlendingStyle::Luminosity => "luminosity",
    }
  }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize, Debug)]
pub enum CompositionStyle {
  SrcIn,
  SrcOut,
  SrcOver,
  SrcAtop,
  DestIn,
  DestOut,
  DestOver,
  DestAtop,
  Copy,
  Lighter,
  Xor,
}

impl FromStr for CompositionStyle {
  type Err = ();

  fn from_str(string: &str) -> Result<CompositionStyle, ()> {
    match string {
      "source-in" => Ok(CompositionStyle::SrcIn),
      "source-out" => Ok(CompositionStyle::SrcOut),
      "source-over" => Ok(CompositionStyle::SrcOver),
      "source-atop" => Ok(CompositionStyle::SrcAtop),
      "destination-in" => Ok(CompositionStyle::DestIn),
      "destination-out" => Ok(CompositionStyle::DestOut),
      "destination-over" => Ok(CompositionStyle::DestOver),
      "destination-atop" => Ok(CompositionStyle::DestAtop),
      "copy" => Ok(CompositionStyle::Copy),
      "lighter" => Ok(CompositionStyle::Lighter),
      "xor" => Ok(CompositionStyle::Xor),
      _ => Err(()),
    }
  }
}

impl CompositionStyle {
  pub fn to_str(&self) -> &str {
    match *self {
      CompositionStyle::SrcIn => "source-in",
      CompositionStyle::SrcOut => "source-out",
      CompositionStyle::SrcOver => "source-over",
      CompositionStyle::SrcAtop => "source-atop",
      CompositionStyle::DestIn => "destination-in",
      CompositionStyle::DestOut => "destination-out",
      CompositionStyle::DestOver => "destination-over",
      CompositionStyle::DestAtop => "destination-atop",
      CompositionStyle::Copy => "copy",
      CompositionStyle::Lighter => "lighter",
      CompositionStyle::Xor => "xor",
    }
  }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize, Debug)]
pub enum CompositionOrBlending {
  Composition(CompositionStyle),
  Blending(BlendingStyle),
}

impl Default for CompositionOrBlending {
  fn default() -> CompositionOrBlending {
    CompositionOrBlending::Composition(CompositionStyle::SrcOver)
  }
}

impl FromStr for CompositionOrBlending {
  type Err = ();

  fn from_str(string: &str) -> Result<CompositionOrBlending, ()> {
    if let Ok(op) = CompositionStyle::from_str(string) {
      return Ok(CompositionOrBlending::Composition(op));
    }

    if let Ok(op) = BlendingStyle::from_str(string) {
      return Ok(CompositionOrBlending::Blending(op));
    }

    Err(())
  }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum FillOrStrokeStyle {
  Color(RGBA),
  LinearGradient(LinearGradientStyle),
  RadialGradient(RadialGradientStyle),
  Surface(SurfaceStyle),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CanvasGradientStop {
  pub offset: f64,
  pub color: RGBA,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct LinearGradientStyle {
  pub x0: f64,
  pub y0: f64,
  pub x1: f64,
  pub y1: f64,
  pub stops: Vec<CanvasGradientStop>,
}

impl LinearGradientStyle {
  pub fn new(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    stops: Vec<CanvasGradientStop>,
  ) -> LinearGradientStyle {
    LinearGradientStyle {
      x0: x0,
      y0: y0,
      x1: x1,
      y1: y1,
      stops: stops,
    }
  }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct RadialGradientStyle {
  pub x0: f64,
  pub y0: f64,
  pub r0: f64,
  pub x1: f64,
  pub y1: f64,
  pub r1: f64,
  pub stops: Vec<CanvasGradientStop>,
}

impl RadialGradientStyle {
  pub fn new(
    x0: f64,
    y0: f64,
    r0: f64,
    x1: f64,
    y1: f64,
    r1: f64,
    stops: Vec<CanvasGradientStop>,
  ) -> RadialGradientStyle {
    RadialGradientStyle {
      x0: x0,
      y0: y0,
      r0: r0,
      x1: x1,
      y1: y1,
      r1: r1,
      stops: stops,
    }
  }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SurfaceStyle {
  pub surface_data: Vec<u8>,
  pub surface_size: Size2D<i32>,
  pub repeat_x: bool,
  pub repeat_y: bool,
}

impl SurfaceStyle {
  pub fn new(
    surface_data: Vec<u8>,
    surface_size: Size2D<i32>,
    repeat_x: bool,
    repeat_y: bool,
  ) -> SurfaceStyle {
    SurfaceStyle {
      surface_data: surface_data,
      surface_size: surface_size,
      repeat_x: repeat_x,
      repeat_y: repeat_y,
    }
  }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize, Debug)]
pub enum LineCapStyle {
  Butt = 0,
  Round = 1,
  Square = 2,
}

impl FromStr for LineCapStyle {
  type Err = ();

  fn from_str(string: &str) -> Result<LineCapStyle, ()> {
    match string {
      "butt" => Ok(LineCapStyle::Butt),
      "round" => Ok(LineCapStyle::Round),
      "square" => Ok(LineCapStyle::Square),
      _ => Err(()),
    }
  }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize, Debug)]
pub enum LineJoinStyle {
  Round = 0,
  Bevel = 1,
  Miter = 2,
}

impl FromStr for LineJoinStyle {
  type Err = ();

  fn from_str(string: &str) -> Result<LineJoinStyle, ()> {
    match string {
      "round" => Ok(LineJoinStyle::Round),
      "bevel" => Ok(LineJoinStyle::Bevel),
      "miter" => Ok(LineJoinStyle::Miter),
      _ => Err(()),
    }
  }
}
