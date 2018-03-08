extern crate cssparser;
extern crate euclid;
extern crate image;
extern crate rustcanvas;

use std::fs::File;
use std::f64::consts::PI;
use std::sync::mpsc::{channel};

use cssparser::{RGBA};
use euclid::{Point2D, Size2D, Rect};
use image::png::{PNGDecoder, PNGEncoder};
use image::{ColorType, DecodingResult, DynamicImage, ImageDecoder, open};
use rustcanvas::{create_canvas, CanvasContextType, FillOrStrokeStyle, CanvasMsg, Canvas2dMsg};

fn main() {
  let canvas = create_canvas(1080, 1980, CanvasContextType::CTX2D);
  let renderer = canvas.ctx;
  let (sender, receiver) = channel::<Vec<u8>>();
  let f1_raw = get_raw("examples/fixtures/6423a9e3-665c-4b4a-aaa4-5b9478c2f150.png");
  let f2_raw = get_raw("examples/fixtures/257bf48a-bf98-4e98-bfe5-410d71ec80b3.png");
  println!("{}", f2_raw.len());
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f1_raw,
      Size2D::new(1080.0, 1920.0),
      Rect::new(Point2D::new(-540.0, -960.0), Size2D::new(1080.0, 1920.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      true
    ))
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f2_raw,
      Size2D::new(1011.0, 825.0),
      Rect::new(Point2D::new(-505.5, -412.5), Size2D::new(1011.0, 825.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1011.0, 825.0)),
      true
    ))
  ).unwrap();
  let canvas_size = Size2D::new(1080.0, 1920.0);
  let size_i32 = canvas_size.to_i32();

  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(Rect::new(Point2D::new(0i32, 0i32), size_i32), canvas_size, sender))
  ).unwrap();

  renderer.send(CanvasMsg::Close).unwrap();

  match receiver.recv() {
    Ok(pixels) => {
      let f = File::create("./test.png").unwrap();
      let png = PNGEncoder::new(f);
      png.encode(&pixels, 1080, 1920, ColorType::RGBA(8)).expect("Write File Error");
    },
    Err(e) => panic!("Recv fail: {:?}", e),
  };
}

fn get_raw(r: &str) -> Vec<u8> {
  match open(r).unwrap() {
    DynamicImage::ImageRgb8(rgb_image) => {
      let mut r = vec![];
      let rgb_image = rgb_image.into_raw();
      let len = rgb_image.len();
      let mut i = 0;
      while i < len {
        r.push(rgb_image[i]);
        r.push(rgb_image[i + 1]);
        r.push(rgb_image[i + 2]);
        r.push(255);
        i += 3;
      };
      r
    },
    DynamicImage::ImageRgba8(rgba_image) => rgba_image.into_raw(),
    _ => panic!("unsupport"),
  }
}
