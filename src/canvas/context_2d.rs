use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;

use app_units::Au;
use cairo::prelude::{Pattern, PatternTrait};
use cairo::{Context, Format, ImageSurface, LineCap, LineJoin, Matrix, Operator};
use cairo::{Filter, Gradient, LinearGradient, MatrixTrait, RadialGradient, SurfacePattern};
use cssparser::RGBA;
use euclid::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use fonts::system_fonts;
use lyon_path::PathEvent;
use num_traits::ToPrimitive;
use pathfinder_font_renderer::{FontContext, FontInstance, GlyphKey, SubpixelOffset};

use super::canvas_trait::*;
use super::paintstate::{Font, PaintState};
use csshelper::SANS_SERIF_FONT_FAMILY;
use fontrenderer::flip_text;

static NEXT_FONT_KEY: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FontKey(usize);

impl FontKey {
  fn new() -> FontKey {
    FontKey(NEXT_FONT_KEY.fetch_add(1, Ordering::SeqCst))
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Context2DError {
  reason: String,
}

impl Context2DError {
  pub fn new(reason: &str) -> Context2DError {
    Context2DError {
      reason: reason.to_owned(),
    }
  }
}

impl Display for Context2DError {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{}", self.reason)
  }
}

impl Error for Context2DError {
  fn description(&self) -> &str {
    self.reason.as_ref()
  }
}

pub struct GlobalFontContext {
  font_context: FontContext<FontKey>,
  font_caches: BTreeMap<String, FontKey>,
}

unsafe impl Send for GlobalFontContext {}
unsafe impl Sync for GlobalFontContext {}

lazy_static! {
  static ref GLOBAL_FONT_CONTEXT: GlobalFontContext = {
    let mut font_context = FontContext::new().unwrap();
    let mut font_caches = BTreeMap::new();
    system_fonts::query_all().into_iter().for_each(|font| {
      let font_property = system_fonts::FontPropertyBuilder::new()
        .family(&font)
        .build();
      let (buffer, _) = system_fonts::get(&font_property).unwrap();
      match font_caches.entry(font) {
        Entry::Occupied(_) => (),
        Entry::Vacant(entry) => {
          let font_key = FontKey::new();
          font_context
            .add_font_from_memory(&font_key, Arc::new(buffer), 0)
            .unwrap();
          entry.insert(font_key);
        }
      }
    });
    GlobalFontContext {
      font_context,
      font_caches,
    }
  };
}

pub struct Context2d {
  pub state: PaintState,
  saved_states: Vec<PaintState>,
  pub cairo_ctx: Context,
}

impl Context2d {
  fn read_pixels(&mut self, _read_rect: Rect<i32>, _canvas_size: Size2D<f64>) -> Vec<u8> {
    let surface = self.cairo_ctx.get_target();
    let image_surface = ImageSurface::from(surface).expect("ImageSurface from surface fail");
    let mut dist: Vec<u8> = vec![];
    image_surface.write_to_png(&mut dist).unwrap();

    dist
  }

  pub fn new(size: Size2D<i32>) -> Result<Context2d, Context2DError> {
    let image_surface = try!(
      ImageSurface::create(Format::ARgb32, size.width, size.height).map_err(|_| Context2DError {
        reason: "Cairo create ImageSurface fail".to_owned()
      })
    );
    let cairo_ctx = Context::new(&image_surface);

    Ok(Context2d {
      state: PaintState::new(),
      saved_states: vec![],
      cairo_ctx,
    })
  }

  pub fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
    let (sender, receiver) = channel::<CanvasMsg>();
    thread::Builder::new()
      .name("CanvasThread".to_owned())
      .spawn(move || {
        let mut painter = Context2d::new(size).expect("Create Context2D fail");
        loop {
          let msg = receiver.recv();
          match msg.expect("CanvasThread recive msg fail") {
            CanvasMsg::Canvas2d(message) => {
              painter
                .handle_canvas2d_msg(message)
                .expect("Handle canvas2d msg error");
            }
            CanvasMsg::Close => break,
            CanvasMsg::FromScript(message) => match message {
              FromScriptMsg::SendPixels(chan) => painter.send_pixels(chan),
            },
          }
        }
      })
      .expect("Thread spawning failed");

    sender
  }

  pub fn handle_canvas2d_msg(&mut self, message: Canvas2dMsg) -> Result<(), Context2DError> {
    match message {
      Canvas2dMsg::FillText(text, x, y, max_width) => self.fill_text(text, x, y, max_width),
      Canvas2dMsg::StrokeText(text, x, y, max_width) => self.stroke_text(text, x, y, max_width),
      Canvas2dMsg::FillRect(ref rect) => self.fill_rect(rect),
      Canvas2dMsg::StrokeRect(ref rect) => self.stroke_rect(rect),
      Canvas2dMsg::ClearRect(ref rect) => Ok(self.clear_rect(rect)),
      Canvas2dMsg::BeginPath => Ok(self.begin_path()),
      Canvas2dMsg::ClosePath => Ok(self.close_path()),
      Canvas2dMsg::Fill => Ok(self.fill()),
      Canvas2dMsg::Stroke => Ok(self.stroke()),
      Canvas2dMsg::Clip => Ok(self.clip()),
      // Canvas2dMsg::IsPointInPath(x, y, fill_rule, chan) => chan
      //   .send(self.is_point_in_path(x, y, fill_rule))
      //   .map_err(|err| Context2DError {
      //     reason: format!("{}", err),
      //   }),
      Canvas2dMsg::DrawImage(imagedata, image_size, dest_rect, source_rect, smoothing_enabled) => {
        self.draw_image(
          imagedata,
          image_size,
          dest_rect,
          source_rect,
          smoothing_enabled,
        )
      }
      Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled) => {
        self.draw_image_self(image_size, dest_rect, source_rect, smoothing_enabled)
      }
      Canvas2dMsg::MoveTo(ref point) => Ok(self.move_to(point)),
      Canvas2dMsg::LineTo(ref point) => Ok(self.line_to(point)),
      Canvas2dMsg::Rect(ref rect) => Ok(self.rect(rect)),
      Canvas2dMsg::QuadraticCurveTo(ref cp, ref pt) => Ok(self.quadratic_curve_to(cp, pt)),
      Canvas2dMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
        Ok(self.bezier_curve_to(cp1, cp2, pt))
      }
      Canvas2dMsg::Arc(ref center, radius, start, end, ccw) => {
        Ok(self.arc(center, radius, start, end, ccw))
      }
      Canvas2dMsg::ArcTo(ref cp1, ref cp2, radius) => Ok(self.arc_to(cp1, cp2, radius)),
      Canvas2dMsg::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) => {
        Ok(self.ellipse(center, radius_x, radius_y, rotation, start, end, ccw))
      }
      Canvas2dMsg::RestoreContext => Ok(self.restore_context_state()),
      Canvas2dMsg::SaveContext => Ok(self.save_context_state()),
      Canvas2dMsg::SetFillStyle(style) => Ok(self.set_fill_style(style)),
      Canvas2dMsg::SetFontStyle(font_rule) => Ok(self.set_font_style(&font_rule)),
      Canvas2dMsg::SetStrokeStyle(style) => Ok(self.set_stroke_style(style)),
      Canvas2dMsg::SetLineWidth(width) => Ok(self.set_line_width(width)),
      Canvas2dMsg::SetLineCap(cap) => Ok(self.set_line_cap(cap)),
      Canvas2dMsg::SetLineJoin(join) => Ok(self.set_line_join(join)),
      Canvas2dMsg::SetMiterLimit(limit) => Ok(self.set_miter_limit(limit)),
      Canvas2dMsg::SetTransform(ref matrix) => Ok(self.set_transform(matrix)),
      Canvas2dMsg::SetGlobalAlpha(alpha) => Ok(self.set_global_alpha(alpha)),
      Canvas2dMsg::SetGlobalComposition(op) => Ok(self.set_global_composition(op)),
      // Canvas2dMsg::GetImageData(dest_rect, canvas_size, chan) => chan
      //   .send(self.image_data(dest_rect, canvas_size))
      //   .map_err(|err| Context2DError {
      //     reason: format!("{}", err),
      //   }),
      Canvas2dMsg::PutImageData(imagedata, offset, image_data_size, dirty_rect) => {
        Ok(self.put_image_data(imagedata, offset, image_data_size, dirty_rect))
      }
      Canvas2dMsg::SetShadowOffsetX(value) => Ok(self.set_shadow_offset_x(value)),
      Canvas2dMsg::SetShadowOffsetY(value) => Ok(self.set_shadow_offset_y(value)),
      Canvas2dMsg::SetShadowBlur(value) => Ok(self.set_shadow_blur(value)),
      Canvas2dMsg::SetShadowColor(color) => Ok(self.set_shadow_color(color)),
      Canvas2dMsg::NotImplement => Ok(()),
      _ => Err(Context2DError {
        reason: "Not Implement Error".to_owned(),
      }),
    }
  }

  pub fn save_context_state(&mut self) {
    self.saved_states.push(self.state.clone());
    self.cairo_ctx.save();
  }

  pub fn restore_context_state(&mut self) {
    if let Some(state) = self.saved_states.pop() {
      mem::replace(&mut self.state, state);
      self.cairo_ctx.restore();
    }
  }

  pub fn fill_text(
    &mut self,
    text: String,
    x: f32,
    y: f32,
    max_width: Option<f32>,
  ) -> Result<(), Context2DError> {
    try!(self.draw_text(text, x, y, max_width));
    self.fill();
    Ok(())
  }

  pub fn stroke_text(
    &mut self,
    text: String,
    x: f32,
    y: f32,
    max_width: Option<f32>,
  ) -> Result<(), Context2DError> {
    try!(self.draw_text(text, x, y, max_width));
    self.stroke();
    Ok(())
  }

  pub fn draw_text(
    &mut self,
    text: String,
    x: f32,
    y: f32,
    max_width: Option<f32>,
  ) -> Result<(), Context2DError> {
    let font = &self.state.font;
    let family = &font.font_family;
    let font_keys = &GLOBAL_FONT_CONTEXT.font_caches;
    let size = &font.font_size;
    let font_key = match font_keys.get(family) {
      Some(f) => f,
      None => try!(font_keys.get(SANS_SERIF_FONT_FAMILY).ok_or(Context2DError {
        reason: "Font fallback error, can not found any default font".to_owned()
      })),
    };
    let instance = FontInstance::new(font_key, Au::from_px(*size as i32));
    let mut offset_x = x;
    let scale = match max_width {
      Some(m) => {
        let total_width = text
          .chars()
          .map(|c| {
            let font_context = &GLOBAL_FONT_CONTEXT.font_context;
            let pos = try!(
              font_context
                .get_char_index(&font_key, c)
                .ok_or(Context2DError {
                  reason: "Pathfinder get char index fail".to_owned()
                })
            );
            let glyph_key = GlyphKey::new(pos, SubpixelOffset(0));
            let glyph_dimensions = try!(
              font_context
                .glyph_dimensions(&instance, &glyph_key, false)
                .map_err(|()| Context2DError {
                  reason: "Pathfinder glyph_dimensions fail".to_owned()
                })
            );
            Ok(glyph_dimensions.advance)
          })
          .sum::<Result<f32, Context2DError>>();
        if try!(total_width) > m {
          m / try!(total_width)
        } else {
          1.0
        }
      }
      None => 1.0,
    };
    for c in text.chars() {
      let font_context = &GLOBAL_FONT_CONTEXT.font_context;
      let pos = try!(
        font_context
          .get_char_index(&font_key, c)
          .ok_or(Context2DError {
            reason: "Get Char index font_context fail".to_owned()
          })
      );
      let glyph_key = GlyphKey::new(pos, SubpixelOffset(0));
      let glyph_dimensions = try!(
        font_context
          .glyph_dimensions(&instance, &glyph_key, false)
          .map_err(|()| Context2DError {
            reason: "Get glyph dimensions fail".to_owned()
          })
      );
      let text_width = glyph_dimensions.size.width;
      let advance = glyph_dimensions.advance;
      let advance_offset = advance / 2.0 * scale;
      offset_x = offset_x + advance_offset;
      let text_width = text_width as f32 * scale;
      let glyph_outline = try!(
        font_context
          .glyph_outline(&instance, &glyph_key)
          .map_err(|()| Context2DError {
            reason: "Pathfinder glyph_outline fail".to_owned()
          })
      );
      glyph_outline
        .iter()
        .map(|e| flip_text(scale)(e))
        .for_each(|f| match f {
          PathEvent::MoveTo(p) => {
            self.move_to(&Point2D::new((p.x + offset_x) as f64, (p.y + y) as f64))
          }
          PathEvent::LineTo(p) => {
            self.line_to(&Point2D::new((p.x + offset_x) as f64, (p.y + y) as f64))
          }
          PathEvent::QuadraticTo(cp, ep) => self.quadratic_curve_to(
            &Point2D::new((cp.x + offset_x) as f64, (cp.y + y) as f64),
            &Point2D::new((ep.x + offset_x) as f64, (ep.y + y) as f64),
          ),
          PathEvent::Close => self.close_path(),
          PathEvent::CubicTo(cp1, cp2, ep) => self.bezier_curve_to(
            &Point2D::new((cp1.x + offset_x) as f64, (cp1.y + y) as f64),
            &Point2D::new((cp2.x + offset_x) as f64, (cp2.y + y) as f64),
            &Point2D::new((ep.x + offset_x) as f64, (ep.y + y) as f64),
          ),
          PathEvent::Arc(c, r, s, e) => self.arc(
            &Point2D::new((c.x + offset_x) as f64, (c.y + y) as f64),
            r.angle_from_x_axis().get() as f64,
            s.get() as f64,
            e.get() as f64,
            false,
          ),
        });
      offset_x = offset_x + advance_offset + text_width;
    }
    Ok(())
  }

  pub fn fill_rect(&self, rect: &Rect<f64>) -> Result<(), Context2DError> {
    if is_zero_size_gradient(&self.state.fill_style) {
      return Ok(()); // Paint nothing if gradient size is zero.
    }

    let draw_rect = Rect::new(
      rect.origin,
      match self.state.fill_style {
        CairoPattern::SurfacePattern(ref _surface) => {
          unimplemented!();
        }
        _ => rect.size,
      },
    );

    if self.need_to_draw_shadow() {
      try!(
        self.draw_with_shadow(&draw_rect, |new_cairo_ctx: &Context| Ok(
          new_cairo_ctx.rectangle(
            draw_rect.origin.x,
            draw_rect.origin.y,
            draw_rect.size.width,
            draw_rect.size.height,
          )
        ))
      );
    } else {
      self.cairo_ctx.rectangle(
        draw_rect.origin.x,
        draw_rect.origin.y,
        draw_rect.size.width,
        draw_rect.size.height,
      );
    }
    self.cairo_ctx.fill();
    Ok(())
  }

  pub fn clear_rect(&self, rect: &Rect<f64>) {
    let operator = self.cairo_ctx.get_operator();
    self.cairo_ctx.set_operator(Operator::Clear);
    self.cairo_ctx.rectangle(
      rect.origin.x,
      rect.origin.y,
      rect.size.width,
      rect.size.height,
    );
    self.cairo_ctx.fill();
    self.cairo_ctx.set_operator(operator);
  }

  pub fn stroke_rect(&self, rect: &Rect<f64>) -> Result<(), Context2DError> {
    if is_zero_size_gradient(&self.state.stroke_style) {
      return Ok(()); // Paint nothing if gradient size is zero.
    }

    if self.need_to_draw_shadow() {
      try!(self.draw_with_shadow(&rect, |new_cairo_ctx: &Context| Ok(
        new_cairo_ctx.rectangle(
          rect.origin.x,
          rect.origin.y,
          rect.size.width,
          rect.size.height,
        )
      )));
    } else if rect.size.width == 0. || rect.size.height == 0. {
    } else {
      self.cairo_ctx.rectangle(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
      );
    }
    self.cairo_ctx.stroke();
    Ok(())
  }

  pub fn begin_path(&self) {
    self.cairo_ctx.new_path();
  }

  pub fn close_path(&self) {
    self.cairo_ctx.close_path()
  }

  pub fn fill(&self) {
    if is_zero_size_gradient(&self.state.fill_style) {
      return; // Paint nothing if gradient size is zero.
    }

    self.cairo_ctx.fill();
  }

  pub fn stroke(&self) {
    if is_zero_size_gradient(&self.state.stroke_style) {
      return; // Paint nothing if gradient size is zero.
    }

    self.cairo_ctx.stroke();
  }

  pub fn clip(&self) {
    self.cairo_ctx.clip();
  }

  pub fn is_point_in_path(&mut self, x: f64, y: f64, _fill_rule: FillRule) -> bool {
    self.cairo_ctx.in_stroke(x, y)
  }

  pub fn draw_image(
    &self,
    image_data: Vec<u8>,
    image_size: Size2D<f64>,
    dest_rect: Rect<f64>,
    source_rect: Rect<f64>,
    smoothing_enabled: bool,
  ) -> Result<(), Context2DError> {
    // We round up the floating pixel values to draw the pixels
    let source_rect = source_rect.ceil();
    // It discards the extra pixels (if any) that won't be painted
    let image_data = crop_image(image_data, image_size, source_rect);
    if self.need_to_draw_shadow() {
      let rect = Rect::new(
        Point2D::new(dest_rect.origin.x, dest_rect.origin.y),
        Size2D::new(dest_rect.size.width, dest_rect.size.height),
      );

      self.draw_with_shadow(&rect, |new_cairo_ctx: &Context| {
        write_image(
          &new_cairo_ctx,
          image_data,
          source_rect.size,
          dest_rect,
          self.state.global_alpha,
          smoothing_enabled,
        )
      })
    } else {
      write_image(
        &self.cairo_ctx,
        image_data,
        source_rect.size,
        dest_rect,
        self.state.global_alpha,
        smoothing_enabled,
      )
    }
  }

  pub fn ellipse(
    &mut self,
    center: &Point2D<f64>,
    radius_x: f64,
    radius_y: f64,
    rotation_angle: f64,
    start_angle: f64,
    end_angle: f64,
    ccw: bool,
  ) {
    let old_martix = self.cairo_ctx.get_matrix();
    self.cairo_ctx.translate(radius_x, radius_y);
    self.cairo_ctx.rotate(rotation_angle);
    self.cairo_ctx.scale(radius_x / radius_y, 1.0);
    self.cairo_ctx.translate(-radius_x, -radius_y);
    self.arc(center, radius_y, start_angle, end_angle, ccw);
    self.cairo_ctx.set_matrix(old_martix);
  }

  pub fn set_fill_style(&self, style: FillOrStrokeStyle) {
    match style {
      FillOrStrokeStyle::Color(rgba) => self.cairo_ctx.set_source_rgba(
        rgba.red as f64 / 255.0f64,
        rgba.green as f64 / 255.0f64,
        rgba.blue as f64 / 255.0f64,
        rgba.alpha as f64 / 255.0f64,
      ),
      _ => {}
    }
  }

  pub fn set_font_style(&mut self, font_style: &str) {
    self.state.font = Font::new(font_style);
  }

  pub fn set_stroke_style(&self, style: FillOrStrokeStyle) {
    match style {
      FillOrStrokeStyle::Color(rgba) => self.cairo_ctx.set_source_rgba(
        rgba.red as f64 / 255.0f64,
        rgba.green as f64 / 255.0f64,
        rgba.blue as f64 / 255.0f64,
        rgba.alpha as f64 / 255.0f64,
      ),
      _ => {}
    }
  }

  pub fn set_line_width(&self, width: f64) {
    self.cairo_ctx.set_line_width(width);
  }

  pub fn set_line_cap(&mut self, cap: LineCapStyle) {
    self.cairo_ctx.set_line_cap(cap.to_cairo_style());
  }

  pub fn set_line_join(&self, join: LineJoinStyle) {
    self.cairo_ctx.set_line_join(join.to_cairo_style());
  }

  pub fn set_miter_limit(&self, limit: f64) {
    self.cairo_ctx.set_miter_limit(limit);
  }

  pub fn set_transform(&mut self, transform: &Transform2D<f64>) {
    self.state.transform = transform.clone();
    self.cairo_ctx.transform(transform.to_cairo_style());
  }

  pub fn set_global_alpha(&mut self, alpha: f64) {
    self.state.global_alpha = alpha;
    self.cairo_ctx.paint_with_alpha(alpha)
  }

  pub fn set_global_composition(&self, op: CompositionOrBlending) {
    self.cairo_ctx.set_operator(op.to_cairo_style());
  }

  pub fn set_shadow_offset_x(&mut self, value: f64) {
    self.state.shadow_offset_x = value;
  }

  pub fn set_shadow_offset_y(&mut self, value: f64) {
    self.state.shadow_offset_y = value;
  }

  pub fn set_shadow_blur(&mut self, value: f64) {
    self.state.shadow_blur = value;
  }

  pub fn set_shadow_color(&mut self, value: RGBA) {
    self.state.shadow_color = value;
  }

  // https://html.spec.whatwg.org/multipage/#when-shadows-are-drawn
  fn need_to_draw_shadow(&self) -> bool {
    self.state.shadow_color.alpha != 0
      && (self.state.shadow_offset_x != 0.0f64 || self.state.shadow_offset_y != 0.0f64
        || self.state.shadow_blur != 0.0f64)
  }

  pub fn draw_with_shadow<F>(
    &self,
    rect: &Rect<f64>,
    draw_shadow_source: F,
  ) -> Result<(), Context2DError>
  where
    F: FnOnce(&Context) -> Result<(), Context2DError>,
  {
    self.cairo_ctx.save();
    let pad = self.state.shadow_blur * 2.0f64;
    let shadow_src_rect = self.state.transform.transform_rect(rect);
    let shadow_surface = try!(
      ImageSurface::create(
        Format::ARgb32,
        (shadow_src_rect.size.width + pad) as i32,
        (shadow_src_rect.size.height + pad) as i32,
      ).map_err(|_| Context2DError {
        reason: "Cairo create ImageSurface in draw_with_shadow fail".to_owned()
      })
    );
    let shadow_ctx = Context::new(&shadow_surface);
    let mut old_pattern = self.cairo_ctx.get_source();
    let surface_pattern = SurfacePattern::create(&shadow_surface);
    self
      .cairo_ctx
      .set_source(&mut Pattern::SurfacePattern(surface_pattern));
    let shadow_color = self.state.shadow_color;
    self.cairo_ctx.set_source_rgba(
      shadow_color.red_f32() as f64,
      shadow_color.green_f32() as f64,
      shadow_color.blue_f32() as f64,
      shadow_color.alpha_f32() as f64,
    );
    self.cairo_ctx.paint();
    // let origin_surface = self.cairo_ctx.get_target();
    // let origin_surface_as_pattern = SurfacePattern::create(&origin_surface);
    // TODO
    // shadow_ctx.mask_surface(&origin_surface_as_pattern, pad, pad)
    // self.cairo_ctx.set_source_surface(shadow_surface, dx - sx + (context->state->shadowOffsetX / fx) - pad + 1.4,
    // dy - sy + (context->state->shadowOffsetY / fy) - pad + 1.4);
    try!(draw_shadow_source(&shadow_ctx));
    self.cairo_ctx.set_source(&mut old_pattern);
    self.cairo_ctx.restore();
    Ok(())
  }

  pub fn draw_image_self(
    &mut self,
    image_size: Size2D<f64>,
    dest_rect: Rect<f64>,
    source_rect: Rect<f64>,
    smoothing_enabled: bool,
  ) -> Result<(), Context2DError> {
    // Reads pixels from source image
    // In this case source and target are the same canvas
    let image_data = self.read_pixels(source_rect.to_i32(), image_size);

    if self.need_to_draw_shadow() {
      let rect = Rect::new(
        Point2D::new(dest_rect.origin.x, dest_rect.origin.y),
        Size2D::new(dest_rect.size.width, dest_rect.size.height),
      );

      self.draw_with_shadow(&rect, |new_cario_ctx: &Context| {
        write_image(
          &new_cario_ctx,
          image_data,
          source_rect.size,
          dest_rect,
          self.state.global_alpha,
          smoothing_enabled,
        )
      })
    } else {
      // Writes on target canvas
      write_image(
        &self.cairo_ctx,
        image_data,
        image_size,
        dest_rect,
        self.state.global_alpha,
        smoothing_enabled,
      )
    }
  }

  pub fn move_to(&self, point: &Point2D<f64>) {
    self.cairo_ctx.move_to(point.x, point.y)
  }

  pub fn line_to(&self, point: &Point2D<f64>) {
    self.cairo_ctx.line_to(point.x, point.y)
  }

  pub fn rect(&self, rect: &Rect<f64>) {
    self.move_to(&Point2D::new(rect.origin.x, rect.origin.y));
    self.line_to(&Point2D::new(
      rect.origin.x + rect.size.width,
      rect.origin.y,
    ));
    self.line_to(&Point2D::new(
      rect.origin.x + rect.size.width,
      rect.origin.y + rect.size.height,
    ));
    self.line_to(&Point2D::new(
      rect.origin.x,
      rect.origin.y + rect.size.height,
    ));
    self.close_path();
  }

  pub fn quadratic_curve_to(&self, cp: &Point2D<f64>, endpoint: &Point2D<f64>) {
    let (x, y) = self.cairo_ctx.get_current_point();
    let cp1x = (x + 2.0f64 * cp.x) / 3.0f64;
    let cp1y = (y + 2.0f64 * cp.y) / 3.0f64;
    let cp2x = (endpoint.x + 2.0f64 * cp.x) / 3.0f64;
    let cp2y = (endpoint.y + 2.0f64 * cp.y) / 3.0f64;
    self
      .cairo_ctx
      .curve_to(cp1x, cp1y, cp2x, cp2y, endpoint.x, endpoint.y);
  }

  pub fn bezier_curve_to(&self, cp1: &Point2D<f64>, cp2: &Point2D<f64>, endpoint: &Point2D<f64>) {
    self
      .cairo_ctx
      .curve_to(cp1.x, cp1.y, cp2.x, cp2.y, endpoint.x, endpoint.y);
  }

  pub fn arc(
    &self,
    center: &Point2D<f64>,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    ccw: bool,
  ) {
    if !ccw {
      self
        .cairo_ctx
        .arc(center.x, center.y, radius, start_angle, end_angle);
    } else {
      self
        .cairo_ctx
        .arc_negative(center.x, center.y, radius, start_angle, end_angle);
    }
  }

  pub fn arc_to(&self, cp1: &Point2D<f64>, cp2: &Point2D<f64>, radius: f64) {
    let (cpx, cpy) = self.cairo_ctx.get_current_point();
    let cp1 = *cp1;
    let cp2 = *cp2;

    if (cpx == cpx && cpy == cpy) || cp1 == cp2 || radius == 0.0 {
      self.line_to(&cp1);
      return;
    }

    // if all three control points lie on a single straight line,
    // connect the first two by a straight line
    let direction = (cp2.x - cp1.x) * (cpy - cp1.y) + (cp2.y - cp1.y) * (cp1.x - cpx);
    if direction == 0.0 {
      self.line_to(&cp1);
      return;
    }

    // otherwise, draw the Arc
    let a2 = (cpx - cp1.x).powi(2) + (cpy - cp1.y).powi(2);
    let b2 = (cp1.x - cp2.x).powi(2) + (cp1.y - cp2.y).powi(2);
    let d = {
      let c2 = (cpx - cp2.x).powi(2) + (cpy - cp2.y).powi(2);
      let cosx = (a2 + b2 - c2) / (2.0 * (a2 * b2).sqrt());
      let sinx = (1.0 - cosx.powi(2)).sqrt();
      radius / ((1.0 - cosx) / sinx)
    };

    // first tangent point
    let anx = (cp1.x - cpx) / a2.sqrt();
    let any = (cp1.y - cpy) / a2.sqrt();
    let tp1 = Point2D::new(cp1.x - anx * d, cp1.y - any * d);

    // second tangent point
    let bnx = (cp1.x - cp2.x) / b2.sqrt();
    let bny = (cp1.y - cp2.y) / b2.sqrt();
    let tp2 = Point2D::new(cp1.x - bnx * d, cp1.y - bny * d);

    // arc center and angles
    let anticlockwise = direction < 0.0;
    let cx = tp1.x + any * radius * if anticlockwise { 1.0 } else { -1.0 };
    let cy = tp1.y - anx * radius * if anticlockwise { 1.0 } else { -1.0 };
    let angle_start = (tp1.y - cy).atan2(tp1.x - cx);
    let angle_end = (tp2.y - cy).atan2(tp2.x - cx);

    self.line_to(&cp1);
    if [cx, cy, angle_start, angle_end]
      .iter()
      .all(|x| x.is_finite())
    {
      self.arc(
        &Point2D::new(cx, cy),
        radius,
        angle_start,
        angle_end,
        anticlockwise,
      );
    }
  }

  // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
  fn put_image_data(
    &mut self,
    imagedata: Vec<u8>,
    offset: Vector2D<f64>,
    image_data_size: Size2D<f64>,
    mut dirty_rect: Rect<f64>,
  ) {
    if image_data_size.width <= 0.0 || image_data_size.height <= 0.0 {
      return;
    }

    debug_assert_eq!(
      image_data_size.width * image_data_size.height * 4.0,
      imagedata.len() as f64
    );

    // Step 1. TODO (neutered data)

    // Step 2.
    if dirty_rect.size.width < 0.0f64 {
      dirty_rect.origin.x += dirty_rect.size.width;
      dirty_rect.size.width = -dirty_rect.size.width;
    }

    if dirty_rect.size.height < 0.0f64 {
      dirty_rect.origin.y += dirty_rect.size.height;
      dirty_rect.size.height = -dirty_rect.size.height;
    }

    // Step 3.
    if dirty_rect.origin.x < 0.0f64 {
      dirty_rect.size.width += dirty_rect.origin.x;
      dirty_rect.origin.x = 0.0f64;
    }

    if dirty_rect.origin.y < 0.0f64 {
      dirty_rect.size.height += dirty_rect.origin.y;
      dirty_rect.origin.y = 0.0f64;
    }

    // Step 4.
    if dirty_rect.max_x() > image_data_size.width {
      dirty_rect.size.width = image_data_size.width - dirty_rect.origin.x;
    }

    if dirty_rect.max_y() > image_data_size.height {
      dirty_rect.size.height = image_data_size.height - dirty_rect.origin.y;
    }

    // 5) If either dirtyWidth or dirtyHeight is negative or zero,
    // stop without affecting any bitmaps
    if dirty_rect.size.width <= 0.0 || dirty_rect.size.height <= 0.0 {
      return;
    }

    // Step 6.
    let dest_rect = dirty_rect.translate(&offset).to_i32();

    // azure_hl operates with integers. We need to cast the image size
    let image_size = image_data_size.to_i32();

    let first_pixel = dest_rect.origin - offset.to_i32();
    let mut src_line = (first_pixel.y * (image_size.width * 4) + first_pixel.x * 4) as usize;

    let mut dest = Vec::with_capacity((dest_rect.size.width * dest_rect.size.height * 4) as usize);

    for _ in 0..dest_rect.size.height {
      let mut src_offset = src_line;
      for _ in 0..dest_rect.size.width {
        let alpha = imagedata[src_offset + 3] as u16;
        // add 127 before dividing for more accurate rounding
        let premultiply_channel = |channel: u8| (((channel as u16 * alpha) + 127) / 255) as u8;
        dest.push(premultiply_channel(imagedata[src_offset + 2]));
        dest.push(premultiply_channel(imagedata[src_offset + 1]));
        dest.push(premultiply_channel(imagedata[src_offset + 0]));
        dest.push(imagedata[src_offset + 3]);
        src_offset += 4;
      }
      src_line += (image_size.width * 4) as usize;
    }

    if let Ok(source_surface) = ImageSurface::create_for_data(
      Box::from(imagedata.as_slice()),
      |d| {
        drop(d);
      },
      Format::ARgb32,
      dest_rect.size.width,
      dest_rect.size.height,
      dest_rect.size.width * 4,
    ) {
      self.cairo_ctx.set_source_surface(
        &source_surface,
        dest_rect.origin.x as f64,
        dest_rect.origin.y as f64,
      );
      self.cairo_ctx.paint();
    }
  }

  pub fn image_data(&mut self, dest_rect: Rect<i32>, canvas_size: Size2D<f64>) -> Vec<u8> {
    self.read_pixels(dest_rect, canvas_size)
  }

  pub fn send_pixels(&mut self, _chan: Sender<Option<Vec<u8>>>) {
    unimplemented!();
  }
}

fn is_zero_size_gradient(pattern: &CairoPattern) -> bool {
  match *pattern {
    CairoPattern::LinearGradient(ref linear) => {
      let (x1, y1, x2, y2) = linear.get_linear_points();
      return x1 == x2 && y1 == y2;
    }
    _ => false,
  }
}

pub trait PointToi32 {
  fn to_i32(&self) -> Point2D<i32>;
}

impl PointToi32 for Point2D<f64> {
  fn to_i32(&self) -> Point2D<i32> {
    Point2D::new(self.x.to_i32().unwrap(), self.y.to_i32().unwrap())
  }
}

pub trait SizeToi32 {
  fn to_i32(&self) -> Size2D<i32>;
}

impl SizeToi32 for Size2D<f64> {
  fn to_i32(&self) -> Size2D<i32> {
    Size2D::new(self.width.to_i32().unwrap(), self.height.to_i32().unwrap())
  }
}

pub trait RectToi32 {
  fn to_i32(&self) -> Rect<i32>;
  fn ceil(&self) -> Rect<f64>;
}

impl RectToi32 for Rect<f64> {
  fn to_i32(&self) -> Rect<i32> {
    Rect::new(
      Point2D::new(
        self.origin.x.to_i32().unwrap(),
        self.origin.y.to_i32().unwrap(),
      ),
      Size2D::new(
        self.size.width.to_i32().unwrap(),
        self.size.height.to_i32().unwrap(),
      ),
    )
  }

  fn ceil(&self) -> Rect<f64> {
    Rect::new(
      Point2D::new(self.origin.x.ceil(), self.origin.y.ceil()),
      Size2D::new(self.size.width.ceil(), self.size.height.ceil()),
    )
  }
}

/// Used by drawImage to get rid of the extra pixels of the image data that
/// won't be copied to the canvas
/// image_data: Color pixel data of the image
/// image_size: Image dimensions
/// crop_rect: It determines the area of the image we want to keep
fn crop_image(image_data: Vec<u8>, _image_size: Size2D<f64>, _crop_rect: Rect<f64>) -> Vec<u8> {
  image_data
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
fn write_image(
  cairo_ctx: &Context,
  image_data: Vec<u8>,
  image_size: Size2D<f64>,
  dest_rect: Rect<f64>,
  global_alpha: f64,
  smoothing_enabled: bool,
) -> Result<(), Context2DError> {
  if image_data.is_empty() {
    return Ok(());
  }

  // From spec https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
  // When scaling up, if the imageSmoothingEnabled attribute is set to true, the user agent should attempt
  // to apply a smoothing algorithm to the image data when it is scaled.
  // Otherwise, the image must be rendered using nearest-neighbor interpolation.
  let filter = if smoothing_enabled {
    Filter::Best
  } else {
    Filter::Fast
  };

  let source_surface = try!(
    ImageSurface::create_from_png(&mut image_data.as_slice()).map_err(|_| Context2DError {
      reason: "Cairo create_from_png fail in write_image fail".to_owned()
    })
  );

  let scale_x = image_size.width / dest_rect.size.width;
  let scale_y = image_size.height / dest_rect.size.height;
  cairo_ctx.set_source_surface(&source_surface, dest_rect.origin.x, dest_rect.origin.y);
  let pattern = cairo_ctx.get_source();
  pattern.set_filter(filter);
  let mut matrix = pattern.get_matrix();
  matrix.scale(scale_x, scale_y);
  pattern.set_matrix(matrix);
  cairo_ctx.paint_with_alpha(global_alpha);
  Ok(())
}

pub trait ToCairoStyle {
  type Target;
  fn to_cairo_style(self) -> Self::Target;
}

impl ToCairoStyle for CompositionOrBlending {
  type Target = Operator;

  fn to_cairo_style(self) -> Operator {
    match self {
      CompositionOrBlending::Composition(op) => op.to_cairo_style(),
      CompositionOrBlending::Blending(op) => op.to_cairo_style(),
    }
  }
}

pub trait ToCairoPattern {
  fn to_cairo_pattern(&self) -> Option<CairoPattern>;
}

impl ToCairoPattern for FillOrStrokeStyle {
  fn to_cairo_pattern(&self) -> Option<CairoPattern> {
    match *self {
      FillOrStrokeStyle::Color(color) => Some(CairoPattern::Color(color)),
      FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
        let gradient = LinearGradient::new(
          linear_gradient_style.x0,
          linear_gradient_style.y0,
          linear_gradient_style.x1,
          linear_gradient_style.y1,
        );
        linear_gradient_style.stops.iter().for_each(|s| {
          gradient.add_color_stop_rgba(
            s.offset,
            s.color.red_f32() as f64,
            s.color.green_f32() as f64,
            s.color.blue_f32() as f64,
            s.color.alpha_f32() as f64,
          );
        });

        Some(CairoPattern::LinearGradient(gradient))
      }
      FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
        let gradient = RadialGradient::new(
          radial_gradient_style.x0,
          radial_gradient_style.y0,
          radial_gradient_style.r0,
          radial_gradient_style.x1,
          radial_gradient_style.y1,
          radial_gradient_style.r1,
        );
        radial_gradient_style.stops.iter().for_each(|s| {
          gradient.add_color_stop_rgba(
            s.offset,
            s.color.red_f32() as f64,
            s.color.green_f32() as f64,
            s.color.blue_f32() as f64,
            s.color.alpha_f32() as f64,
          );
        });

        Some(CairoPattern::RadialGradient(gradient))
      }
      FillOrStrokeStyle::Surface(ref surface_style) => {
        let image_surface = ImageSurface::create_for_data(
          Box::from(surface_style.surface_data.as_slice()),
          |d| {
            drop(d);
          },
          Format::ARgb32,
          surface_style.surface_size.width,
          surface_style.surface_size.height,
          surface_style.surface_size.width * 4,
        ).expect("Create image surface pattern fail");
        Some(CairoPattern::SurfacePattern(SurfacePattern::create(
          &image_surface,
        )))
      }
    }
  }
}

impl ToCairoStyle for LineCapStyle {
  type Target = LineCap;

  fn to_cairo_style(self) -> LineCap {
    match self {
      LineCapStyle::Butt => LineCap::Butt,
      LineCapStyle::Round => LineCap::Round,
      LineCapStyle::Square => LineCap::Square,
    }
  }
}

impl ToCairoStyle for LineJoinStyle {
  type Target = LineJoin;

  fn to_cairo_style(self) -> LineJoin {
    match self {
      LineJoinStyle::Round => LineJoin::Round,
      LineJoinStyle::Bevel => LineJoin::Bevel,
      LineJoinStyle::Miter => LineJoin::Miter,
    }
  }
}

impl ToCairoStyle for CompositionStyle {
  type Target = Operator;

  fn to_cairo_style(self) -> Operator {
    match self {
      CompositionStyle::SrcIn => Operator::In,
      CompositionStyle::SrcOut => Operator::Out,
      CompositionStyle::SrcOver => Operator::Over,
      CompositionStyle::SrcAtop => Operator::Atop,
      CompositionStyle::DestIn => Operator::DestIn,
      CompositionStyle::DestOut => Operator::DestOut,
      CompositionStyle::DestOver => Operator::DestOver,
      CompositionStyle::DestAtop => Operator::DestAtop,
      CompositionStyle::Copy => Operator::Source,
      CompositionStyle::Lighter => Operator::Add,
      CompositionStyle::Xor => Operator::Xor,
    }
  }
}

impl ToCairoStyle for BlendingStyle {
  type Target = Operator;

  fn to_cairo_style(self) -> Operator {
    match self {
      BlendingStyle::Multiply => Operator::Multiply,
      BlendingStyle::Screen => Operator::Screen,
      BlendingStyle::Overlay => Operator::Overlay,
      BlendingStyle::Darken => Operator::Darken,
      BlendingStyle::Lighten => Operator::Lighten,
      BlendingStyle::ColorDodge => Operator::ColorDodge,
      BlendingStyle::ColorBurn => Operator::ColorBurn,
      BlendingStyle::HardLight => Operator::HardLight,
      BlendingStyle::SoftLight => Operator::SoftLight,
      BlendingStyle::Difference => Operator::Difference,
      BlendingStyle::Exclusion => Operator::Exclusion,
      BlendingStyle::Hue => Operator::HslHue,
      BlendingStyle::Saturation => Operator::HslSaturation,
      BlendingStyle::Color => Operator::HslColor,
      BlendingStyle::Luminosity => Operator::HslLuminosity,
    }
  }
}

impl ToCairoStyle for Transform2D<f64> {
  type Target = Matrix;

  fn to_cairo_style(self) -> Matrix {
    Matrix {
      xx: self.m11,
      xy: self.m12,
      yx: self.m21,
      yy: self.m22,
      x0: self.m31,
      y0: self.m32,
    }
  }
}

pub fn byte_swap(data: &mut [u8]) {
  let length = data.len();
  // FIXME(rust #27741): Range::step_by is not stable yet as of this writing.
  let mut i = 0;
  while i < length {
    let r = data[i + 2];
    data[i + 2] = data[i + 0];
    data[i + 0] = r;
    i += 4;
  }
}

#[cfg(test)]
mod context_2d_test {
  use super::Context2d;
  use euclid::Size2D;

  #[test]
  fn new_context_2d_check() {
    Context2d::new(Size2D::new(1920, 1080));
  }
}
