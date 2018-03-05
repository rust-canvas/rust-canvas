use std::ptr::{null_mut};
use std::sync::{Arc};

use azure::azure_hl::{DrawTarget, SurfaceFormat};
use cgl;
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use euclid::{Size2D};
use gleam::gl;
use glutin::{Api, GlRequest, HeadlessRendererBuilder, GlContext};
use io_surface;
use skia::gl_rasterization_context::{GLRasterizationContext};
use skia::gl_context::{GLContext as SkiaGLContext, PlatformDisplayData};

#[allow(non_upper_case_globals)]
const kCGLOGLPVersion_3_2_Core: cgl::CGLPixelFormatAttribute = 0x3200;

pub fn get_draw_target(size: Size2D<i32>) -> DrawTarget {
  let headless_gl = HeadlessRendererBuilder::new(size.width as u32, size.height as u32)
    .with_gl(GlRequest::Latest)
    .build_strict()
    .unwrap();
  unsafe {
    headless_gl.make_current().unwrap();
  };
  let gl_instance = match headless_gl.get_api() {
    Api::OpenGl => unsafe {
      println!("Current use OpenGl");
      gl::GlFns::load_with(|symbol| headless_gl.get_proc_address(symbol) as *const _)
    },
    Api::OpenGlEs => unsafe {
      println!("Current use OpenGlEs");
      gl::GlesFns::load_with(|symbol| headless_gl.get_proc_address(symbol) as *const _)
    },
    Api::WebGl => panic!("WebGl is unimplement"),
  };
  println!("Current OpenGL Version: {:?}", gl_instance.get_string(gl::VERSION));
  // borrow from https://github.com/emilio/rust-offscreen-rendering-context/blob/master/src/platform/with_cgl/native_gl_context.rs
  let mut attributes = [
    cgl::kCGLPFAOpenGLProfile, kCGLOGLPVersion_3_2_Core,
    0
  ];
  let mut pixel_format: cgl::CGLPixelFormatObj = null_mut();
  let mut pix_count = 0;
  unsafe {
    if cgl::CGLChoosePixelFormat(attributes.as_mut_ptr(), &mut pixel_format, &mut pix_count) != 0 {
      panic!("CGLChoosePixelFormat");
    }

    if pix_count == 0 {
      panic!("No pixel formats available");
    }
  }

  let pdd = PlatformDisplayData { pixel_format };
  let gl_ctx = SkiaGLContext::new(gl_instance, pdd, size).expect("Fail to create glx_ctx");
  unsafe {
    let width_key: CFString = TCFType::wrap_under_get_rule(io_surface::kIOSurfaceWidth);
    let width_value: CFNumber = CFNumber::from(size.width);

    let height_key: CFString = TCFType::wrap_under_get_rule(io_surface::kIOSurfaceHeight);
    let height_value: CFNumber = CFNumber::from(size.height);

    let bytes_per_row_key: CFString =
      TCFType::wrap_under_get_rule(io_surface::kIOSurfaceBytesPerRow);
    let bytes_per_row_value: CFNumber = CFNumber::from(size.width * 4);

    let bytes_per_elem_key: CFString =
      TCFType::wrap_under_get_rule(io_surface::kIOSurfaceBytesPerElement);
    let bytes_per_elem_value: CFNumber = CFNumber::from(4);

    let is_global_key: CFString =
      TCFType::wrap_under_get_rule(io_surface::kIOSurfaceIsGlobal);
    let is_global_value = CFBoolean::true_value();

    let surface = io_surface::new(&CFDictionary::from_CFType_pairs(&[
      (width_key.as_CFType(), width_value.as_CFType()),
      (height_key.as_CFType(), height_value.as_CFType()),
      (bytes_per_row_key.as_CFType(), bytes_per_row_value.as_CFType()),
      (bytes_per_elem_key.as_CFType(), bytes_per_elem_value.as_CFType()),
      (is_global_key.as_CFType(), is_global_value.as_CFType()),
    ]));
    let gl_rasterization_context = GLRasterizationContext::new(gl_ctx, surface.as_concrete_TypeRef(), size).unwrap();
    DrawTarget::new_with_gl_rasterization_context(Arc::new(gl_rasterization_context), SurfaceFormat::B8G8R8A8)
  }
}
