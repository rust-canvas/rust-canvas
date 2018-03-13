use std::ffi::{CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::{Arc};

use azure::azure_hl::{DrawTarget, SurfaceFormat};
use skia::gl_context::{GLContext, GLRasterizationContext, PlatformDisplayData};
use euclid::{Size2D};
use gleam::gl;
use glx;
use x11;
use x11::xlib::*;

pub fn get_draw_target(size: Size2D<i32>) -> DrawTarget {
  let dpy = unsafe { XOpenDisplay(0 as *mut c_char) as *mut glx::types::Display  };
  let mut attributes = [
    glx::DRAWABLE_TYPE as c_int, glx::PIXMAP_BIT as c_int,
    glx::X_RENDERABLE as c_int, 1,
    glx::RENDER_TYPE as c_int, glx::RGBA_BIT as c_int,
    0 as c_int
  ];
  let mut config_count : c_int = 0;

  let visual_info = unsafe {
    glx::ChooseVisual(dpy,
                        XDefaultScreen(dpy as *mut Display),
                        attributes.as_mut_ptr()) as *mut x11::xlib::XVisualInfo
  };

  let fb_configs = unsafe {
    glx::ChooseFBConfig(dpy,
                        XDefaultScreen(dpy as *mut Display),
                        attributes.as_mut_ptr(),
                        &mut config_count)
  };

  if fb_configs.is_null() {
    panic!("glx::ChooseFBConfig");
  }

  let mut visual_id = glx::NONE as c_int;
  for i in 0..(config_count as isize) {
    unsafe {
      let config = *fb_configs.offset(i);
      let mut drawable_type : c_int = 0;
      // NOTE: glx's `Success` is unreachable from bindings, but it's defined to 0
      // TODO: Check if this conditional is neccesary:
      //   Actually this gets the drawable type and checks if
      //   contains PIXMAP_BIT, which should be true due to the attributes
      //   in glx::ChooseFBConfig
      //
      //   It's in Gecko's code, so may there be an implementation which returns bad
      //   configurations?
      if glx::GetFBConfigAttrib(dpy, config, glx::DRAWABLE_TYPE as c_int, &mut drawable_type) != 0
        || (drawable_type & (glx::PIXMAP_BIT as c_int) == 0) {
        continue;
      }

      if glx::GetFBConfigAttrib(dpy, config, glx::VISUAL_ID as c_int, &mut visual_id) != 0
        || visual_id == 0 {
        continue;
      }
    }
    break;
  }

  if visual_id == 0 {
    panic!("We don't have any config with visuals");
  }

  let pixmap = unsafe {
    let screen = XDefaultScreenOfDisplay(dpy as *mut _);
    let (_, depth) = get_visual_and_depth(screen, visual_id as VisualID).expect("Get visual and depth fail");
    XCreatePixmap(dpy as *mut _,
                  XRootWindowOfScreen(screen),
                  size.width as c_uint,
                  size.height as c_uint,
                  depth as c_uint)
  };

  let pdd = PlatformDisplayData {
    display: dpy as *mut x11::xlib::_XDisplay, visual_info
  };

  let gl = unsafe {
    gl::GlFns::load_with(|symbol| {
      let addr = CString::new(symbol.as_bytes()).unwrap();
      let addr = addr.as_ptr();
      glx::GetProcAddress(addr as *const u8) as *const _
    })
  };
  let gl_ctx = GLContext::new(gl, pdd, size).expect("Create gl_ctx fail");
  let gl_rasterization_context = Arc::new(GLRasterizationContext::new(gl_ctx, pixmap, size).expect("Create gl_rasterization_context fail"));
  DrawTarget::new_with_gl_rasterization_context(gl_rasterization_context, SurfaceFormat::B8G8R8A8)
}

unsafe fn get_visual_and_depth(s: *mut Screen, id: VisualID) -> Result<(*mut Visual, c_int), &'static str> {
  for d in 0..((*s).ndepths as isize) {
    let depth_info : *mut Depth = (*s).depths.offset(d);
    for v in 0..((*depth_info).nvisuals as isize) {
      let visual : *mut Visual = (*depth_info).visuals.offset(v);
      if (*visual).visualid == id {
        return Ok((visual, (*depth_info).depth));
      }
    }
  }
  Err("Visual not on screen")
}
