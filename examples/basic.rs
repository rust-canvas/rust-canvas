extern crate cssparser;
extern crate euclid;
extern crate image;
extern crate rustcanvas;

use std::f64::consts::PI;
use std::fs::{File};
use std::io::prelude::*;
use std::sync::mpsc::{channel};

use cssparser::{RGBA};
use euclid::{Point2D, Size2D, Rect};
use rustcanvas::{create_canvas, CanvasContextType, FillOrStrokeStyle, CanvasMsg, Canvas2dMsg};

fn main() {
  let canvas = create_canvas(1920, 1080, CanvasContextType::CTX2D);
  let renderer = canvas.ctx;
  let (sender, receiver) = channel::<Vec<u8>>();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineWidth(10.0))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetStrokeStyle(FillOrStrokeStyle::Color(RGBA::new(66, 165, 245, 255))))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::MoveTo(Point2D::new(100.0, 100.0)))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::LineTo(Point2D::new(600.0, 600.0)))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::MoveTo(Point2D::new(700.0, 200.0)))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Stroke)).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetStrokeStyle(FillOrStrokeStyle::Color(RGBA::new(244, 143, 177, 255))))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::BezierCurveTo(Point2D::new(760.0, 300.0), Point2D::new(920.0, 425.0), Point2D::new(1100.0, 200.0)))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Stroke)).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(FillOrStrokeStyle::Color(RGBA::new(233, 193, 127, 255))))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Arc(Point2D::new(700.0, 600.0), 400.0, 0.0, 2.0 * PI as f32, false))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Fill)).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(FillOrStrokeStyle::Color(RGBA::new(0, 0, 0, 255))))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFontStyle("200px \"PingFang TC\"".to_string()))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::FillText("哈哈".to_string(), 1000.0, 800.0, Some(200.0)))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(FillOrStrokeStyle::Color(RGBA::new(244, 143, 177, 255))))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::FillText("二豆".to_string(), 300.0, 800.0, None))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetStrokeStyle(FillOrStrokeStyle::Color(RGBA::new(66, 165, 245, 255))))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::StrokeText("来呀打我啊".to_string(), 300.0, 400.0, None))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFontStyle("200px \"Monaco\"".to_string()))).unwrap();
  renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::FillText("Hello Moto".to_string(), 500.0, 700.0, None))).unwrap();
  let canvas_size = Size2D::new(1920.0, 1080.0);
  let size_i32 = canvas_size.to_i32();

  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(Rect::new(Point2D::new(0i32, 0i32), size_i32), canvas_size, sender))
  ).unwrap();

  renderer.send(CanvasMsg::Close).unwrap();

  match receiver.recv() {
    Ok(pixels) => {
      let mut f = File::create("./test.png").unwrap();
      f.write(&pixels).unwrap();
    },
    Err(e) => panic!("Recv fail: {:?}", e),
  };

}
