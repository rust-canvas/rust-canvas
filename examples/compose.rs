extern crate cssparser;
extern crate euclid;
extern crate image;
extern crate rustcanvas;

use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc::{channel};

use cssparser::{RGBA};
use euclid::{Point2D, Size2D, Rect};
use rustcanvas::{
  create_canvas,
  CanvasContextType,
  CanvasMsg,
  Canvas2dMsg,
  CompositionOrBlending,
  CompositionStyle,
  FillOrStrokeStyle,
};

fn main() {
  let canvas = create_canvas(1080, 1920, CanvasContextType::CTX2D);
  let renderer = canvas.ctx;
  let (sender, receiver) = channel::<Vec<u8>>();
  let f1_raw = get_raw("6423a9e3-665c-4b4a-aaa4-5b9478c2f150.png");
  let f2_raw = get_raw("257bf48a-bf98-4e98-bfe5-410d71ec80b3.png");
  let f3_raw = get_raw("63611baa-2888-46f3-a2dd-bfb54d7f4482.png");
  let f4_raw = get_raw("a68be70f-df59-494b-8e44-5e7a176afc31.png");
  let f5_raw = get_raw("ff2aaa4a-4996-4230-a3a3-761903878464.png");
  let f6_raw = get_raw("7f307cfe-1aa4-41ce-884b-762892f8bf18.png");
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::SetGlobalComposition(
      CompositionOrBlending::Composition(CompositionStyle::SrcOver)
    ))
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f1_raw,
      Size2D::new(1080.0, 1920.0),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      true
    ))
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f2_raw,
      Size2D::new(1011.0, 825.0),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1011.0, 825.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1011.0, 825.0)),
      true
    ))
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f3_raw,
      Size2D::new(1125.0, 2463.0),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1125.0, 2463.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1125.0, 2463.0)),
      true
    ))
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f4_raw,
      Size2D::new(1080.0, 1920.0),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      true
    ))
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f5_raw,
      Size2D::new(1080.0, 1920.0),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1080.0, 1920.0)),
      true
    ))
  ).unwrap();
  let offset_x = 593.65 - 97.5;
  let offset_y = 1380.4 - 35.1 + 61.425;
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::SetFillStyle(FillOrStrokeStyle::Color(RGBA::new(0, 0, 0, 255)))
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::SetFontStyle(String::from("normal  195px \"PingFang TC\""))
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("雷"), offset_x, offset_y - 286.65, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("厉"), offset_x, offset_y - 95.54, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("风"), offset_x, offset_y + 95.55, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("行"), offset_x, offset_y + 286.65, None)
    )
  ).unwrap();
  let offset_x = 735.0;
  let offset_y = 1360.0;

  let canvas_size = Size2D::new(1080.0, 1920.0);
  let size_i32 = canvas_size.to_i32();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::SetFontStyle(String::from("normal  51px \"PingFang TC\""))
    )
  ).unwrap();

  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("有"), offset_x, offset_y - 350.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("执"), offset_x, offset_y - 300.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("着"), offset_x, offset_y - 250.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("的"), offset_x, offset_y - 200.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("梦"), offset_x, offset_y - 150.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("也"), offset_x, offset_y - 100.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("有"), offset_x, offset_y - 50.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("洒"), offset_x, offset_y, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("脱"), offset_x, offset_y + 50.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("的"), offset_x, offset_y + 100.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("生"), offset_x, offset_y + 150.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("活"), offset_x, offset_y + 200.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("和"), offset_x, offset_y + 250.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("红"), offset_x, offset_y + 300.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("包"), offset_x, offset_y + 350.0, None)
    )
  ).unwrap();

  let offset_x = 805.0;
  let offset_y = 1170.0;
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::SetFontStyle(String::from("normal  90px \"PingFang TC\""))
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("谨"), offset_x, offset_y - 132.3, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("贺"), offset_x, offset_y - 44.1, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("新"), offset_x, offset_y + 44.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("春"), offset_x, offset_y + 132.3, None)
    )
  ).unwrap();

  let offset_x = 910.0;
  let offset_y = 1130.0;
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("雷"), offset_x, offset_y - 90.0, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("佳"), offset_x, offset_y, None)
    )
  ).unwrap();
  renderer.send(
    CanvasMsg::Canvas2d(
      Canvas2dMsg::FillText(String::from("音"), offset_x, offset_y + 90.0, None)
    )
  ).unwrap();

  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f6_raw,
      Size2D::new(96.0, 136.0),
      Rect::new(Point2D::new( 964.3750419449117 - 48.0, 1791.263570829159 - 68.0), Size2D::new(96.0, 136.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(96.0, 136.0)),
      true
    ))
  ).unwrap();

  let f7_raw = get_raw("1517989108553_e6b7e74f-3728-4514-ac88-29fa31a10e9b.png");
  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
      f7_raw,
      Size2D::new(270.0, 270.0),
      Rect::new(Point2D::new(165.3658536585367 - 135.0, 1754.8997233368143 - 135.0), Size2D::new(270.0, 270.0)),
      Rect::new(Point2D::new(0.0, 0.0), Size2D::new(270.0, 270.0)),
      true
    ))
  ).unwrap();

  renderer.send(
    CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(Rect::new(Point2D::new(0i32, 0i32), size_i32), canvas_size, sender))
  ).unwrap();

  renderer.send(CanvasMsg::Close).unwrap();

  match receiver.recv() {
    Ok(pixels) => {
      let mut f = File::create("./compose.png").unwrap();
      f.write(&pixels).unwrap();
    },
    Err(e) => panic!("Recv fail: {:?}", e),
  };
}

fn get_raw(r: &str) -> Vec<u8> {
  let mut p = String::new();
  p.push_str("examples/fixtures/");
  p.push_str(r);
  let mut f = File::open(p).unwrap();
  let mut content = vec![];
  f.read_to_end(&mut content).unwrap();
  content
}
