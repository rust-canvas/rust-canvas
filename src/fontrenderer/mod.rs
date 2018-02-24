use euclid::{Transform2D};
use lyon_path::{PathEvent};

pub fn flip_text(scale: f32) -> Box<Fn(PathEvent) -> PathEvent> {
  let flip = move |event: PathEvent| -> PathEvent {
    let text_transform: Transform2D<f32> = Transform2D::from_row_major_array([
      1.0 * scale, 0.0,
      0.0, -1.0,
      0.0, 0.0
    ]);
    match event {
      PathEvent::MoveTo(p) => PathEvent::MoveTo(text_transform.transform_point(&p)),
      PathEvent::LineTo(p) => PathEvent::LineTo(text_transform.transform_point(&p)),
      PathEvent::QuadraticTo(ctrl, to) => {
        PathEvent::QuadraticTo(
          text_transform.transform_point(&ctrl),
          text_transform.transform_point(&to)
        )
      },
      PathEvent::CubicTo(ctrl1, ctrl2, to) => {
        PathEvent::CubicTo(
          text_transform.transform_point(&ctrl1),
          text_transform.transform_point(&ctrl2),
          text_transform.transform_point(&to)
        )
      },
      PathEvent::Arc(center, radius, start, end) => {
        PathEvent::Arc(
          text_transform.transform_point(&center),
          text_transform.transform_vector(&radius),
          start,
          end
        )
      },
      e => e
    }
  };
  Box::new(flip)
}
