use std::cell::{RefCell};
use std::collections::{BTreeMap};
use std::collections::btree_map::{Entry};
use std::mem;
use std::sync::{Arc};
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread;

use app_units::Au;
use azure::azure_hl::JoinStyle;
use azure::azure_hl::GradientStop;
use azure::azure_hl::{Pattern, DrawTarget, SurfaceFormat, DrawSurfaceOptions};
use azure::azure_hl::{AntialiasMode, CompositionOp, Color, DrawOptions, Filter, ColorPattern};
use azure::azure_hl::{LinearGradientPattern, ExtendMode, RadialGradientPattern, SurfacePattern};
use azure::azure_hl::{PathBuilder, CapStyle, StrokeOptions};
use azure::{AzFloat};
use euclid::{Rect, Point2D, Vector2D, Transform2D, Size2D};
use fonts::system_fonts;
use lyon_path::{PathEvent};
use num_traits::ToPrimitive;
use pathfinder_font_renderer::{FontContext, FontInstance, GlyphKey, SubpixelOffset};

use fontrenderer::{flip_text};
use csshelper::{SANS_SERIF_FONT_FAMILY};
use super::canvas_trait::*;
use super::paintstate::{Font, PaintState};
use super::get_target::{get_draw_target};

static NEXT_FONT_KEY: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct FontKey(usize);

impl FontKey {
  fn new() -> FontKey {
    FontKey(NEXT_FONT_KEY.fetch_add(1, Ordering::SeqCst))
  }
}

pub struct Context2d<'a> {
  pub state: PaintState<'a>,
  saved_states: Vec<PaintState<'a>>,
  drawtarget: DrawTarget,
  path_builder: PathBuilder,
  font_context: RefCell<FontContext<FontKey>>,
  font_caches: BTreeMap<String, FontKey>,
}

impl <'a> Context2d<'a> {
  fn read_pixels(&self, read_rect: Rect<i32>, canvas_size: Size2D<f64>) -> Vec<u8>{
    let canvas_size = canvas_size.to_i32();
    let canvas_rect = Rect::new(Point2D::new(0i32, 0i32), canvas_size);
    let src_read_rect = canvas_rect.intersection(&read_rect).unwrap_or(Rect::zero());

    let mut image_data = vec![];
    if src_read_rect.is_empty() || canvas_size.width <= 0 && canvas_size.height <= 0 {
      return image_data;
    }

    let data_surface = self.drawtarget.snapshot().get_data_surface();
    let mut src_data = Vec::new();
    data_surface.with_data(|element| { src_data = element.to_vec(); });
    let stride = data_surface.stride();

    //start offset of the copyable rectangle
    let mut src = (src_read_rect.origin.y * stride + src_read_rect.origin.x * 4) as usize;
    //copy the data to the destination vector
    for _ in 0..src_read_rect.size.height {
      let row = &src_data[src .. src + (4 * src_read_rect.size.width) as usize];
      image_data.extend_from_slice(row);
      src += stride as usize;
    }

    image_data
  }

  pub fn new(size: Size2D<i32>) -> Context2d<'a> {
    let drawtarget = get_draw_target(size);
    let path_builder = drawtarget.create_path_builder();

    let mut ctx = Context2d {
      state: PaintState::new(),
      saved_states: vec![],
      drawtarget,
      path_builder,
      font_context: RefCell::new(FontContext::new().expect("init FontContext fail")),
      font_caches: BTreeMap::new(),
    };
    system_fonts::query_all().into_iter().for_each(|font| {
      let font_property = system_fonts::FontPropertyBuilder::new().family(&font).build();
      let (buffer, _) = system_fonts::get(&font_property).unwrap();
      ctx.add_font_instance(buffer, font).unwrap();
    });

    ctx
  }

  pub fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
    let (sender, receiver) = channel::<CanvasMsg>();
    thread::Builder::new().name("CanvasThread".to_owned()).spawn(move || {
      let mut painter = Context2d::new(size);
      loop {
        let msg = receiver.recv();
        match msg.expect("CanvasThread recive msg fail") {
          CanvasMsg::Canvas2d(message) => {
            painter.handle_canvas2d_msg(message);
          },
          CanvasMsg::Close => break,
          CanvasMsg::FromScript(message) => {
            match message {
              FromScriptMsg::SendPixels(chan) => {
                painter.send_pixels(chan)
              }
            }
          }
        }
      }
    }).expect("Thread spawning failed");

    sender
  }

  fn handle_canvas2d_msg(&mut self, message: Canvas2dMsg) {
    match message {
      Canvas2dMsg::FillText(text, x, y, max_width) => self.fill_text(text, x, y, max_width),
      Canvas2dMsg::StrokeText(text, x, y, max_width) => self.stroke_text(text, x, y, max_width),
      Canvas2dMsg::FillRect(ref rect) => self.fill_rect(rect),
      Canvas2dMsg::StrokeRect(ref rect) => self.stroke_rect(rect),
      Canvas2dMsg::ClearRect(ref rect) => self.clear_rect(rect),
      Canvas2dMsg::BeginPath => self.begin_path(),
      Canvas2dMsg::ClosePath => self.close_path(),
      Canvas2dMsg::Fill => self.fill(),
      Canvas2dMsg::Stroke => self.stroke(),
      Canvas2dMsg::Clip => self.clip(),
      Canvas2dMsg::IsPointInPath(x, y, fill_rule, chan) => {
        self.is_point_in_path(x, y, fill_rule, chan)
      },
      Canvas2dMsg::DrawImage(imagedata, image_size, dest_rect, source_rect,
                              smoothing_enabled) => {
        self.draw_image(imagedata, image_size, dest_rect, source_rect, smoothing_enabled)
      }
      Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled) => {
        self.draw_image_self(image_size, dest_rect, source_rect, smoothing_enabled)
      }
      Canvas2dMsg::MoveTo(ref point) => self.move_to(point),
      Canvas2dMsg::LineTo(ref point) => self.line_to(point),
      Canvas2dMsg::Rect(ref rect) => self.rect(rect),
      Canvas2dMsg::QuadraticCurveTo(ref cp, ref pt) => {
        self.quadratic_curve_to(cp, pt)
      }
      Canvas2dMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
        self.bezier_curve_to(cp1, cp2, pt)
      }
      Canvas2dMsg::Arc(ref center, radius, start, end, ccw) => {
        self.arc(center, radius, start, end, ccw)
      }
      Canvas2dMsg::ArcTo(ref cp1, ref cp2, radius) => {
        self.arc_to(cp1, cp2, radius)
      }
      Canvas2dMsg::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) => {
        self.ellipse(center, radius_x, radius_y, rotation, start, end, ccw)
      }
      Canvas2dMsg::RestoreContext => self.restore_context_state(),
      Canvas2dMsg::SaveContext => self.save_context_state(),
      Canvas2dMsg::SetFillStyle(style) => self.set_fill_style(style),
      Canvas2dMsg::SetFontStyle(font_rule) => self.set_font_style(&font_rule),
      Canvas2dMsg::SetStrokeStyle(style) => self.set_stroke_style(style),
      Canvas2dMsg::SetLineWidth(width) => self.set_line_width(width),
      Canvas2dMsg::SetLineCap(cap) => self.set_line_cap(cap),
      Canvas2dMsg::SetLineJoin(join) => self.set_line_join(join),
      Canvas2dMsg::SetMiterLimit(limit) => self.set_miter_limit(limit),
      Canvas2dMsg::SetTransform(ref matrix) => self.set_transform(matrix),
      Canvas2dMsg::SetGlobalAlpha(alpha) => self.set_global_alpha(alpha),
      Canvas2dMsg::SetGlobalComposition(op) => self.set_global_composition(op),
      Canvas2dMsg::GetImageData(dest_rect, canvas_size, chan)
          => self.image_data(dest_rect, canvas_size, chan),
      Canvas2dMsg::PutImageData(imagedata, offset, image_data_size, dirty_rect)
          => self.put_image_data(imagedata, offset, image_data_size, dirty_rect),
      Canvas2dMsg::SetShadowOffsetX(value) => self.set_shadow_offset_x(value),
      Canvas2dMsg::SetShadowOffsetY(value) => self.set_shadow_offset_y(value),
      Canvas2dMsg::SetShadowBlur(value) => self.set_shadow_blur(value),
      Canvas2dMsg::SetShadowColor(ref color) => self.set_shadow_color(color.to_azure_style()),
      Canvas2dMsg::NotImplement => { },
    }
  }

  fn add_font_instance(&mut self, bytes: Vec<u8>, family_name: String) -> Result<(), ()> {
    match self.font_caches.entry(family_name) {
      Entry::Occupied(_) => Ok(()),
      Entry::Vacant(entry) => {
        let font_key = FontKey::new();
        match self.font_context.borrow_mut().add_font_from_memory(&font_key, Arc::new(bytes), 0) {
          Ok(_) => {
            entry.insert(font_key);
            Ok(())
          },
          Err(e) => panic!(e),
        }
      }
    }
  }

  fn save_context_state(&mut self) {
    self.saved_states.push(self.state.clone());
  }

  fn restore_context_state(&mut self) {
    if let Some(state) = self.saved_states.pop() {
      mem::replace(&mut self.state, state);
      self.drawtarget.set_transform(&self.state.transform);
      self.drawtarget.pop_clip();
    }
  }

  fn fill_text(&mut self, text: String, x: f32, y: f32, max_width: Option<f32>) {
    self.draw_text(text, x, y, max_width);
    self.fill();
  }

  fn stroke_text(&mut self, text: String, x: f32, y: f32, max_width: Option<f32>) {
    self.draw_text(text, x, y, max_width);
    self.stroke();
  }

  fn draw_text(&mut self, text: String, x: f32, y: f32, max_width: Option<f32>) {
    let font = &self.state.font;
    let family = &font.font_family;
    let font_keys = &self.font_caches;
    let size = &font.font_size;
    let font_key = match font_keys.get(family) {
      Some(f) => f,
      None => font_keys.get(SANS_SERIF_FONT_FAMILY).expect("Get fallback font fail"),
    };
    let instance = FontInstance::new(font_key, Au::from_px(* size as i32));
    let mut offset_x = x;
    let scale = match max_width {
      Some(m) => {
        let total_width = text.chars().map(|c| {
          let font_context = self.font_context.borrow();
          let pos = font_context.get_char_index(&font_key, c).expect("Get Char index font_context fail");
          let glyph_key = GlyphKey::new(pos, SubpixelOffset(0));
          let glyph_dimensions = font_context.glyph_dimensions(&instance, &glyph_key, false).expect("Get glyph dimensions fail");
          glyph_dimensions.advance
        }).sum::<f32>();
        if total_width > m {
          m / total_width
        } else {
          1.0
        }
      },
      None => 1.0,
    };
    text.chars().for_each(|c| {
      let font_context = self.font_context.borrow();
      let pos = font_context.get_char_index(&font_key, c).expect("Get Char index font_context fail");
      let glyph_key = GlyphKey::new(pos, SubpixelOffset(0));
      let glyph_dimensions = font_context.glyph_dimensions(&instance, &glyph_key, false).expect("Get glyph dimensions fail");
      let text_width = glyph_dimensions.size.width;
      let advance = glyph_dimensions.advance;
      let advance_offset = advance / 2.0 * scale;
      offset_x = offset_x + advance_offset;
      let text_width = text_width as f32 * scale;
      let glyph_outline = font_context.glyph_outline(&instance, &glyph_key).expect("Glyph outline fail");
      glyph_outline.iter()
        .map(|e| flip_text(scale)(e))
        .for_each(|f| match f {
          PathEvent::MoveTo(p) => self.move_to(
            &Point2D::new(p.x + offset_x, p.y + y)
          ),
          PathEvent::LineTo(p) => self.line_to(&Point2D::new(p.x + offset_x, p.y + y)),
          PathEvent::QuadraticTo(cp, ep) => self.quadratic_curve_to(
            &Point2D::new(cp.x + offset_x, cp.y + y), &Point2D::new(ep.x + offset_x, ep.y + y)
          ),
          PathEvent::Close => self.close_path(),
          PathEvent::CubicTo(cp1, cp2, ep) => self.bezier_curve_to(
            &Point2D::new(cp1.x + offset_x, cp1.y + y),
            &Point2D::new(cp2.x + offset_x, cp2.y + y),
            &Point2D::new(ep.x + offset_x, ep.y + y)
          ),
          PathEvent::Arc(c, r, s, e) => self.arc(
            &Point2D::new(c.x + offset_x, c.y + y),
            r.angle_from_x_axis().get(), s.get(), e.get(), false
          )
        });
      offset_x = offset_x + advance_offset + text_width;
    });
  }

  fn fill_rect(&self, rect: &Rect<f32>) {
    if is_zero_size_gradient(&self.state.fill_style) {
      return; // Paint nothing if gradient size is zero.
    }

    let draw_rect = Rect::new(rect.origin,
      match self.state.fill_style {
        Pattern::Surface(ref surface) => {
          let surface_size = surface.size();
          match (surface.repeat_x, surface.repeat_y) {
            (true, true) => rect.size,
            (true, false) => Size2D::new(rect.size.width, surface_size.height as f32),
            (false, true) => Size2D::new(surface_size.width as f32, rect.size.height),
            (false, false) => Size2D::new(surface_size.width as f32, surface_size.height as f32),
          }
        },
        _ => rect.size,
      }
    );

    if self.need_to_draw_shadow() {
      self.draw_with_shadow(&draw_rect, |new_draw_target: &DrawTarget| {
        new_draw_target.fill_rect(&draw_rect, self.state.fill_style.to_pattern_ref(),
                                  Some(&self.state.draw_options));
      });
    } else {
      self.drawtarget.fill_rect(&draw_rect, self.state.fill_style.to_pattern_ref(),
                                Some(&self.state.draw_options));
    }
  }

  fn clear_rect(&self, rect: &Rect<f32>) {
    self.drawtarget.clear_rect(rect);
  }

  fn stroke_rect(&self, rect: &Rect<f32>) {
    if is_zero_size_gradient(&self.state.stroke_style) {
      return; // Paint nothing if gradient size is zero.
    }

    if self.need_to_draw_shadow() {
      self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
        new_draw_target.stroke_rect(rect, self.state.stroke_style.to_pattern_ref(),
                                    &self.state.stroke_opts, &self.state.draw_options);
      });
    } else if rect.size.width == 0. || rect.size.height == 0. {
      let cap = match self.state.stroke_opts.line_join {
        JoinStyle::Round => CapStyle::Round,
        _ => CapStyle::Butt
      };

      let stroke_opts =
          StrokeOptions::new(self.state.stroke_opts.line_width,
                              self.state.stroke_opts.line_join,
                              cap,
                              self.state.stroke_opts.miter_limit,
                              self.state.stroke_opts.mDashPattern);
      self.drawtarget.stroke_line(rect.origin, rect.bottom_right(),
                                  self.state.stroke_style.to_pattern_ref(),
                                  &stroke_opts, &self.state.draw_options);
    } else {
      self.drawtarget.stroke_rect(rect, self.state.stroke_style.to_pattern_ref(),
                                  &self.state.stroke_opts, &self.state.draw_options);
    }
  }

  fn begin_path(&mut self) {
    self.path_builder = self.drawtarget.create_path_builder()
  }

  fn close_path(&self) {
    self.path_builder.close()
  }

  fn fill(&self) {
    if is_zero_size_gradient(&self.state.fill_style) {
      return; // Paint nothing if gradient size is zero.
    }

    self.drawtarget.fill(&self.path_builder.finish(),
                          self.state.fill_style.to_pattern_ref(),
                          &self.state.draw_options);
  }

  fn stroke(&self) {
    if is_zero_size_gradient(&self.state.stroke_style) {
      return; // Paint nothing if gradient size is zero.
    }

    self.drawtarget.stroke(&self.path_builder.finish(),
                            self.state.stroke_style.to_pattern_ref(),
                            &self.state.stroke_opts,
                            &self.state.draw_options);
  }

  fn clip(&self) {
    self.drawtarget.push_clip(&self.path_builder.finish());
  }

  fn is_point_in_path(&mut self, x: f64, y: f64,
                      _fill_rule: FillRule, chan: Sender<bool>) {
    let path = self.path_builder.finish();
    let result = path.contains_point(x, y, &self.state.transform);
    self.path_builder = path.copy_to_builder();
    chan.send(result).unwrap();
  }

  fn draw_image(&self, image_data: Vec<u8>, image_size: Size2D<f64>,
                dest_rect: Rect<f64>, source_rect: Rect<f64>, smoothing_enabled: bool) {
      // We round up the floating pixel values to draw the pixels
    let source_rect = source_rect.ceil();
    // It discards the extra pixels (if any) that won't be painted
    let image_data = crop_image(image_data, image_size, source_rect);

    if self.need_to_draw_shadow() {
      let rect = Rect::new(Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                            Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32));

      self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
        write_image(&new_draw_target, image_data, source_rect.size, dest_rect,
                    smoothing_enabled, self.state.draw_options.composition,
                    self.state.draw_options.alpha);
      });
    } else {
      write_image(&self.drawtarget, image_data, source_rect.size, dest_rect,
                  smoothing_enabled, self.state.draw_options.composition,
                  self.state.draw_options.alpha);
    }
  }

  fn create_draw_target_for_shadow(&self, source_rect: &Rect<f32>) -> DrawTarget {
    let draw_target = self.drawtarget.create_similar_draw_target(&Size2D::new(source_rect.size.width as i32,
                                                                              source_rect.size.height as i32),
                                                                  self.drawtarget.get_format());
    let matrix = Transform2D::identity()
      .pre_translate(-source_rect.origin.to_vector().cast().unwrap())
      .pre_mul(&self.state.transform);
    draw_target.set_transform(&matrix);
    draw_target
  }

  fn ellipse(&mut self,
          center: &Point2D<AzFloat>,
          radius_x: AzFloat,
          radius_y: AzFloat,
          rotation_angle: AzFloat,
          start_angle: AzFloat,
          end_angle: AzFloat,
          ccw: bool) {
    self.path_builder.ellipse(*center, radius_x, radius_y, rotation_angle, start_angle, end_angle, ccw);
  }

  fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
    if let Some(pattern) = style.to_azure_pattern(&self.drawtarget) {
      self.state.fill_style = pattern
    }
  }

  fn set_font_style(&mut self, font_style: &str) {
    self.state.font = Font::new(font_style);
  }

  fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
    if let Some(pattern) = style.to_azure_pattern(&self.drawtarget) {
      self.state.stroke_style = pattern
    }
  }

  fn set_line_width(&mut self, width: f32) {
    self.state.stroke_opts.line_width = width;
  }

  fn set_line_cap(&mut self, cap: LineCapStyle) {
    self.state.stroke_opts.line_cap = cap.to_azure_style();
  }

  fn set_line_join(&mut self, join: LineJoinStyle) {
    self.state.stroke_opts.line_join = join.to_azure_style();
  }

  fn set_miter_limit(&mut self, limit: f32) {
    self.state.stroke_opts.miter_limit = limit;
  }

  fn set_transform(&mut self, transform: &Transform2D<f32>) {
    self.state.transform = transform.clone();
    self.drawtarget.set_transform(transform)
  }

  fn set_global_alpha(&mut self, alpha: f32) {
    self.state.draw_options.alpha = alpha;
  }

  fn set_global_composition(&mut self, op: CompositionOrBlending) {
    self.state.draw_options.set_composition_op(op.to_azure_style());
  }

  fn set_shadow_offset_x(&mut self, value: f64) {
    self.state.shadow_offset_x = value;
  }

  fn set_shadow_offset_y(&mut self, value: f64) {
    self.state.shadow_offset_y = value;
  }

  fn set_shadow_blur(&mut self, value: f64) {
    self.state.shadow_blur = value;
  }

  fn set_shadow_color(&mut self, value: Color) {
    self.state.shadow_color = value;
  }

  // https://html.spec.whatwg.org/multipage/#when-shadows-are-drawn
  fn need_to_draw_shadow(&self) -> bool {
    self.state.shadow_color.a != 0.0f32 &&
    (self.state.shadow_offset_x != 0.0f64 ||
      self.state.shadow_offset_y != 0.0f64 ||
      self.state.shadow_blur != 0.0f64)
  }

  fn draw_with_shadow<F>(&self, rect: &Rect<f32>, draw_shadow_source: F)
      where F: FnOnce(&DrawTarget)
  {
    let shadow_src_rect = self.state.transform.transform_rect(rect);
    let new_draw_target = self.create_draw_target_for_shadow(&shadow_src_rect);
    draw_shadow_source(&new_draw_target);
    self.drawtarget.draw_surface_with_shadow(new_draw_target.snapshot(),
                                            &Point2D::new(shadow_src_rect.origin.x as AzFloat,
                                                          shadow_src_rect.origin.y as AzFloat),
                                            &self.state.shadow_color,
                                            &Vector2D::new(self.state.shadow_offset_x as AzFloat,
                                                          self.state.shadow_offset_y as AzFloat),
                                            (self.state.shadow_blur / 2.0f64) as AzFloat,
                                            self.state.draw_options.composition);
  }

  fn draw_image_self(&self, image_size: Size2D<f64>,
                      dest_rect: Rect<f64>, source_rect: Rect<f64>,
                      smoothing_enabled: bool) {
    // Reads pixels from source image
    // In this case source and target are the same canvas
    let image_data = self.read_pixels(source_rect.to_i32(), image_size);

    if self.need_to_draw_shadow() {
      let rect = Rect::new(Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                            Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32));

      self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
        write_image(&new_draw_target, image_data, source_rect.size, dest_rect,
                    smoothing_enabled, self.state.draw_options.composition,
                    self.state.draw_options.alpha);
      });
    } else {
      // Writes on target canvas
      write_image(&self.drawtarget, image_data, image_size, dest_rect,
                  smoothing_enabled, self.state.draw_options.composition,
                  self.state.draw_options.alpha);
    }
  }

  fn move_to(&self, point: &Point2D<AzFloat>) {
    self.path_builder.move_to(*point)
  }

  fn line_to(&self, point: &Point2D<AzFloat>) {
    self.path_builder.line_to(*point)
  }

  fn rect(&self, rect: &Rect<f32>) {
    self.path_builder.move_to(Point2D::new(rect.origin.x, rect.origin.y));
    self.path_builder.line_to(Point2D::new(rect.origin.x + rect.size.width, rect.origin.y));
    self.path_builder.line_to(Point2D::new(rect.origin.x + rect.size.width,
                                            rect.origin.y + rect.size.height));
    self.path_builder.line_to(Point2D::new(rect.origin.x, rect.origin.y + rect.size.height));
    self.path_builder.close();
  }

  fn quadratic_curve_to(&self,
                          cp: &Point2D<AzFloat>,
                          endpoint: &Point2D<AzFloat>) {
    self.path_builder.quadratic_curve_to(cp, endpoint)
  }

  fn bezier_curve_to(&self,
                        cp1: &Point2D<AzFloat>,
                        cp2: &Point2D<AzFloat>,
                        endpoint: &Point2D<AzFloat>) {
    self.path_builder.bezier_curve_to(cp1, cp2, endpoint)
  }

  fn arc(&self,
            center: &Point2D<AzFloat>,
            radius: AzFloat,
            start_angle: AzFloat,
            end_angle: AzFloat,
            ccw: bool) {
    self.path_builder.arc(*center, radius, start_angle, end_angle, ccw)
  }

  fn arc_to(&self,
                cp1: &Point2D<AzFloat>,
                cp2: &Point2D<AzFloat>,
                radius: AzFloat) {
    let cp0 = self.path_builder.get_current_point();
    let cp1 = *cp1;
    let cp2 = *cp2;

    if (cp0.x == cp1.x && cp0.y == cp1.y) || cp1 == cp2 || radius == 0.0 {
      self.line_to(&cp1);
      return;
    }

    // if all three control points lie on a single straight line,
    // connect the first two by a straight line
    let direction = (cp2.x - cp1.x) * (cp0.y - cp1.y) + (cp2.y - cp1.y) * (cp1.x - cp0.x);
    if direction == 0.0 {
      self.line_to(&cp1);
      return;
    }

    // otherwise, draw the Arc
    let a2 = (cp0.x - cp1.x).powi(2) + (cp0.y - cp1.y).powi(2);
    let b2 = (cp1.x - cp2.x).powi(2) + (cp1.y - cp2.y).powi(2);
    let d = {
      let c2 = (cp0.x - cp2.x).powi(2) + (cp0.y - cp2.y).powi(2);
      let cosx = (a2 + b2 - c2) / (2.0 * (a2 * b2).sqrt());
      let sinx = (1.0 - cosx.powi(2)).sqrt();
      radius / ((1.0 - cosx) / sinx)
    };

    // first tangent point
    let anx = (cp1.x - cp0.x) / a2.sqrt();
    let any = (cp1.y - cp0.y) / a2.sqrt();
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

    self.line_to(&tp1);
    if [cx, cy, angle_start, angle_end].iter().all(|x| x.is_finite()) {
      self.arc(&Point2D::new(cx, cy), radius,
                angle_start, angle_end, anticlockwise);
    }
  }

  // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
  fn put_image_data(&mut self, imagedata: Vec<u8>,
                    offset: Vector2D<f64>,
                    image_data_size: Size2D<f64>,
                    mut dirty_rect: Rect<f64>) {
    if image_data_size.width <= 0.0 || image_data_size.height <= 0.0 {
      return
    }

    assert_eq!(image_data_size.width * image_data_size.height * 4.0, imagedata.len() as f64);

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
      return
    }

    // Step 6.
    let dest_rect = dirty_rect.translate(&offset).to_i32();

    // azure_hl operates with integers. We need to cast the image size
    let image_size = image_data_size.to_i32();

    let first_pixel = dest_rect.origin - offset.to_i32();
    let mut src_line = (first_pixel.y * (image_size.width * 4) + first_pixel.x * 4) as usize;

    let mut dest =
      Vec::with_capacity((dest_rect.size.width * dest_rect.size.height * 4) as usize);

    for _ in 0 .. dest_rect.size.height {
      let mut src_offset = src_line;
      for _ in 0 .. dest_rect.size.width {
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

    if let Some(source_surface) = self.drawtarget.create_source_surface_from_data(
            &dest,
            dest_rect.size,
            dest_rect.size.width * 4,
            SurfaceFormat::B8G8R8A8) {
      self.drawtarget.copy_surface(source_surface,
                                    Rect::new(Point2D::new(0, 0), dest_rect.size),
                                    dest_rect.origin);
    }
  }

  fn image_data(&self, dest_rect: Rect<i32>, canvas_size: Size2D<f64>, chan: Sender<Vec<u8>>) {
    let mut dest_data = self.read_pixels(dest_rect, canvas_size);

    // bgra -> rgba
    byte_swap(&mut dest_data);
    chan.send(dest_data).expect("Send image_data fail");
  }

  fn send_pixels(&mut self, chan: Sender<Option<Vec<u8>>>) {
    self.drawtarget.snapshot().get_data_surface().with_data(|element| {
      chan.send(Some(element.into())).expect("Send pixels fail");
    })
  }
}

fn is_zero_size_gradient(pattern: &Pattern) -> bool {
  if let &Pattern::LinearGradient(ref gradient) = pattern {
    if gradient.is_zero_size() {
      return true;
    }
  }
  false
}

pub trait PointToi32 {
  fn to_i32(&self) -> Point2D<i32>;
}

impl PointToi32 for Point2D<f64> {
  fn to_i32(&self) -> Point2D<i32> {
    Point2D::new(self.x.to_i32().unwrap(),
                  self.y.to_i32().unwrap())
  }
}

pub trait SizeToi32 {
    fn to_i32(&self) -> Size2D<i32>;
}

impl SizeToi32 for Size2D<f64> {
  fn to_i32(&self) -> Size2D<i32> {
    Size2D::new(self.width.to_i32().unwrap(),
                self.height.to_i32().unwrap())
  }
}

pub trait RectToi32 {
  fn to_i32(&self) -> Rect<i32>;
  fn ceil(&self) -> Rect<f64>;
}

impl RectToi32 for Rect<f64> {
  fn to_i32(&self) -> Rect<i32> {
    Rect::new(Point2D::new(self.origin.x.to_i32().unwrap(),
                            self.origin.y.to_i32().unwrap()),
              Size2D::new(self.size.width.to_i32().unwrap(),
                          self.size.height.to_i32().unwrap()))
  }

  fn ceil(&self) -> Rect<f64> {
    Rect::new(Point2D::new(self.origin.x.ceil(),
                            self.origin.y.ceil()),
              Size2D::new(self.size.width.ceil(),
                          self.size.height.ceil()))
  }

}

/// Used by drawImage to get rid of the extra pixels of the image data that
/// won't be copied to the canvas
/// image_data: Color pixel data of the image
/// image_size: Image dimensions
/// crop_rect: It determines the area of the image we want to keep
fn crop_image(image_data: Vec<u8>,
              image_size: Size2D<f64>,
              crop_rect: Rect<f64>) -> Vec<u8>{
    // We're going to iterate over a pixel values array so we need integers
  let crop_rect = crop_rect.to_i32();
  let image_size = image_size.to_i32();
  // Assuming 4 bytes per pixel and row-major order for storage
  // (consecutive elements in a pixel row of the image are contiguous in memory)
  let stride = image_size.width * 4;
  let image_bytes_length = image_size.height * image_size.width * 4;
  let crop_area_bytes_length = crop_rect.size.height * crop_rect.size.width * 4;
  // If the image size is less or equal than the crop area we do nothing
  if image_bytes_length <= crop_area_bytes_length {
    return image_data;
  }

  let mut new_image_data = Vec::new();
  let mut src = (crop_rect.origin.y * stride + crop_rect.origin.x * 4) as usize;
  for _ in 0..crop_rect.size.height {
    let row = &image_data[src .. src + (4 * crop_rect.size.width) as usize];
    new_image_data.extend_from_slice(row);
    src += stride as usize;
  }
  new_image_data
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
fn write_image(draw_target: &DrawTarget,
              mut image_data: Vec<u8>,
              image_size: Size2D<f64>,
              dest_rect: Rect<f64>,
              smoothing_enabled: bool,
              composition_op: CompositionOp,
              global_alpha: f32) {
  if image_data.is_empty() {
    return
  }
  let image_rect = Rect::new(Point2D::zero(), image_size);
  // rgba -> bgra
  byte_swap(&mut image_data);

  // From spec https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
  // When scaling up, if the imageSmoothingEnabled attribute is set to true, the user agent should attempt
  // to apply a smoothing algorithm to the image data when it is scaled.
  // Otherwise, the image must be rendered using nearest-neighbor interpolation.
  let filter = if smoothing_enabled {
    Filter::Linear
  } else {
    Filter::Point
  };
  // azure_hl operates with integers. We need to cast the image size
  let image_size = image_size.to_i32();

  if let Some(source_surface) =
    draw_target.create_source_surface_from_data(&mut image_data,
                                                image_size,
                                                image_size.width * 4,
                                                SurfaceFormat::B8G8R8A8) {
    let draw_surface_options = DrawSurfaceOptions::new(filter, true);
    let draw_options = DrawOptions::new(global_alpha, composition_op, AntialiasMode::Subpixel);

    draw_target.draw_surface(source_surface,
                              dest_rect.to_azure_style(),
                              image_rect.to_azure_style(),
                              draw_surface_options,
                              draw_options);
  }
}

pub trait ToAzureStyle {
  type Target;
  fn to_azure_style(self) -> Self::Target;
}

impl ToAzureStyle for Rect<f64> {
  type Target = Rect<AzFloat>;

  fn to_azure_style(self) -> Rect<AzFloat> {
    Rect::new(Point2D::new(self.origin.x as AzFloat, self.origin.y as AzFloat),
              Size2D::new(self.size.width as AzFloat, self.size.height as AzFloat))
  }
}

impl ToAzureStyle for CompositionOrBlending {
  type Target = CompositionOp;

  fn to_azure_style(self) -> CompositionOp {
    match self {
      CompositionOrBlending::Composition(op) => op.to_azure_style(),
      CompositionOrBlending::Blending(op) => op.to_azure_style(),
    }
  }
}

pub trait ToAzurePattern {
  fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern>;
}

impl ToAzurePattern for FillOrStrokeStyle {
  fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern> {
    match *self {
      FillOrStrokeStyle::Color(ref color) => {
        Some(Pattern::Color(ColorPattern::new(color.to_azure_style())))
      },
      FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
        let gradient_stops: Vec<GradientStop> = linear_gradient_style.stops.iter().map(|s| {
          GradientStop {
            offset: s.offset as AzFloat,
            color: s.color.to_azure_style()
          }
        }).collect();

        Some(Pattern::LinearGradient(LinearGradientPattern::new(
            &Point2D::new(linear_gradient_style.x0 as AzFloat, linear_gradient_style.y0 as AzFloat),
            &Point2D::new(linear_gradient_style.x1 as AzFloat, linear_gradient_style.y1 as AzFloat),
            drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
            &Transform2D::identity())))
      },
      FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
        let gradient_stops: Vec<GradientStop> = radial_gradient_style.stops.iter().map(|s| {
          GradientStop {
            offset: s.offset as AzFloat,
            color: s.color.to_azure_style()
          }
        }).collect();

        Some(Pattern::RadialGradient(RadialGradientPattern::new(
          &Point2D::new(radial_gradient_style.x0 as AzFloat, radial_gradient_style.y0 as AzFloat),
          &Point2D::new(radial_gradient_style.x1 as AzFloat, radial_gradient_style.y1 as AzFloat),
          radial_gradient_style.r0 as AzFloat, radial_gradient_style.r1 as AzFloat,
          drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
          &Transform2D::identity())))
      },
      FillOrStrokeStyle::Surface(ref surface_style) => {
        drawtarget.create_source_surface_from_data(&surface_style.surface_data,
                                                    surface_style.surface_size,
                                                    surface_style.surface_size.width * 4,
                                                    SurfaceFormat::B8G8R8A8)
                  .map(|source_surface| {
            Pattern::Surface(SurfacePattern::new(
              source_surface.azure_source_surface,
              surface_style.repeat_x,
              surface_style.repeat_y,
              &Transform2D::identity()))
            })
      }
    }
  }
}

impl ToAzureStyle for LineCapStyle {
  type Target = CapStyle;

  fn to_azure_style(self) -> CapStyle {
    match self {
      LineCapStyle::Butt => CapStyle::Butt,
      LineCapStyle::Round => CapStyle::Round,
      LineCapStyle::Square => CapStyle::Square,
    }
  }
}

impl ToAzureStyle for LineJoinStyle {
  type Target = JoinStyle;

  fn to_azure_style(self) -> JoinStyle {
    match self {
      LineJoinStyle::Round => JoinStyle::Round,
      LineJoinStyle::Bevel => JoinStyle::Bevel,
      LineJoinStyle::Miter => JoinStyle::Miter,
    }
  }
}

impl ToAzureStyle for CompositionStyle {
  type Target = CompositionOp;

  fn to_azure_style(self) -> CompositionOp {
    match self {
      CompositionStyle::SrcIn    => CompositionOp::In,
      CompositionStyle::SrcOut   => CompositionOp::Out,
      CompositionStyle::SrcOver  => CompositionOp::Over,
      CompositionStyle::SrcAtop  => CompositionOp::Atop,
      CompositionStyle::DestIn   => CompositionOp::DestIn,
      CompositionStyle::DestOut  => CompositionOp::DestOut,
      CompositionStyle::DestOver => CompositionOp::DestOver,
      CompositionStyle::DestAtop => CompositionOp::DestAtop,
      CompositionStyle::Copy     => CompositionOp::Source,
      CompositionStyle::Lighter  => CompositionOp::Add,
      CompositionStyle::Xor      => CompositionOp::Xor,
    }
  }
}

impl ToAzureStyle for BlendingStyle {
  type Target = CompositionOp;

  fn to_azure_style(self) -> CompositionOp {
    match self {
      BlendingStyle::Multiply   => CompositionOp::Multiply,
      BlendingStyle::Screen     => CompositionOp::Screen,
      BlendingStyle::Overlay    => CompositionOp::Overlay,
      BlendingStyle::Darken     => CompositionOp::Darken,
      BlendingStyle::Lighten    => CompositionOp::Lighten,
      BlendingStyle::ColorDodge => CompositionOp::ColorDodge,
      BlendingStyle::ColorBurn  => CompositionOp::ColorBurn,
      BlendingStyle::HardLight  => CompositionOp::HardLight,
      BlendingStyle::SoftLight  => CompositionOp::SoftLight,
      BlendingStyle::Difference => CompositionOp::Difference,
      BlendingStyle::Exclusion  => CompositionOp::Exclusion,
      BlendingStyle::Hue        => CompositionOp::Hue,
      BlendingStyle::Saturation => CompositionOp::Saturation,
      BlendingStyle::Color      => CompositionOp::Color,
      BlendingStyle::Luminosity => CompositionOp::Luminosity,
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
  use euclid::{Size2D};
  use super::{Context2d};

  #[test]
  fn new_context_2d_check() {
    Context2d::new(Size2D::new(1920, 1080));
  }
}
