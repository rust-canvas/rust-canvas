use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_uint};
use std::ptr;
use std::str;
use std::sync::{Arc};

use azure::azure_hl::{DrawTarget, SurfaceFormat};
use euclid::{Size2D};
use gleam::gl;
use glx;
use skia::gl_rasterization_context::{GLRasterizationContext};
use skia::gl_context::{GLContext as SkiaGLContext, PlatformDisplayData};
use x11::xlib;

#[derive(Copy, Clone)]
pub struct GLXDisplayInfo {
  pub display: *mut xlib::Display,
  pub configs: Option<glx::types::GLXFBConfig>,
  visual_info: *mut xlib::XVisualInfo,
}

#[derive(Copy, Clone)]
pub enum NativeDisplay {
  GLX(GLXDisplayInfo),
}

impl NativeDisplay {
  pub fn new(display: *mut xlib::Display) -> NativeDisplay {
    // FIXME(pcwalton): It would be more robust to actually have the compositor pass the
    // visual.
    let (compositor_visual_info, configs) =
      NativeDisplay::compositor_visual_info(display);

    NativeDisplay::GLX(GLXDisplayInfo {
      display: display,
      visual_info: compositor_visual_info,
      configs,
    })
  }

  /// Chooses the compositor visual info using the same algorithm that the compositor uses.
  ///
  /// FIXME(pcwalton): It would be more robust to actually have the compositor pass the visual.
  fn compositor_visual_info(display: *mut xlib::Display)
                            -> (*mut xlib::XVisualInfo, Option<glx::types::GLXFBConfig>) {
    // If display is null, we'll assume we are going to be rendering
    // in headless mode without X running.
    if display == ptr::null_mut() {
      return (ptr::null_mut(), None);
    }

    unsafe {
      let fbconfig_attributes = [
        glx::DOUBLEBUFFER as i32, 0,
        glx::DRAWABLE_TYPE as i32, glx::PIXMAP_BIT as i32 | glx::WINDOW_BIT as i32,
        glx::BIND_TO_TEXTURE_RGBA_EXT as i32, 1,
        glx::RENDER_TYPE as i32, glx::RGBA_BIT as i32,
        glx::ALPHA_SIZE as i32, 8,
        0
      ];

      let screen = xlib::XDefaultScreen(display);
      let mut number_of_configs = 0;
      let configs = glx::ChooseFBConfig(mem::transmute(display),
                                        screen,
                                        fbconfig_attributes.as_ptr(),
                                        &mut number_of_configs);
      NativeDisplay::get_compatible_configuration(display, configs, number_of_configs)
    }
  }

  fn get_compatible_configuration(display: *mut xlib::Display,
                                  configs: *mut glx::types::GLXFBConfig,
                                  number_of_configs: i32)
                                  -> (*mut xlib::XVisualInfo, Option<glx::types::GLXFBConfig>) {
    unsafe {
      if number_of_configs == 0 {
        panic!("glx::ChooseFBConfig returned no configurations.");
      }

      if !NativeDisplay::need_to_find_32_bit_depth_visual(display) {
        let config = *configs.offset(0);
        let visual = glx::GetVisualFromFBConfig(display as *mut glx::types::Display, config) as *mut xlib::XVisualInfo;

        xlib::XFree(configs as *mut _);
        return (visual, Some(config));
      }

      // NVidia (and AMD/ATI) drivers have RGBA configurations that use 24-bit
      // XVisual, not capable of representing an alpha-channel in Pixmap form,
      // so we look for the configuration with a full set of 32 bits.
      for i in 0..number_of_configs as isize {
        let config = *configs.offset(i);
        let visual =
          glx::GetVisualFromFBConfig(display as *mut glx::types::Display, config) as *mut xlib::XVisualInfo;
        if (*visual).depth == 24 {
          xlib::XFree(configs as *mut _);
          return (visual, Some(config));
        }
        xlib::XFree(visual as *mut _);
      }

      xlib::XFree(configs as *mut _);
      panic!("Could not find 32-bit visual.");
    }
  }

  fn need_to_find_32_bit_depth_visual(display: *mut xlib::Display) -> bool {
    unsafe {
      let glx_vendor = glx::GetClientString(mem::transmute(display), glx::VENDOR as i32);
      if glx_vendor == ptr::null() {
        panic!("Could not determine GLX vendor.");
      }
      let glx_vendor =
        str::from_utf8(CStr::from_ptr(glx_vendor).to_bytes())
          .ok()
          .expect("GLX client vendor string not in UTF-8 format.")
          .to_string()
          .to_ascii_lowercase();
      glx_vendor.contains("nvidia") || glx_vendor.contains("ati")
    }
  }

  pub fn platform_display_data(&self) -> PlatformDisplayData {
    match *self {
      NativeDisplay::GLX(info) => {
        PlatformDisplayData {
          display: info.display,
          visual_info: info.visual_info,
        }
      }
    }
  }
}

unsafe impl Send for NativeDisplay {}

pub fn get_draw_target(size: Size2D<i32>) -> DrawTarget {
  let gl_instance = unsafe {
    gl::GlFns::load_with(|symbol| {
      let addr = CString::new(symbol.as_bytes()).unwrap();
      let addr = addr.as_ptr();
      glx::GetProcAddress(addr as *const u8) as *const _
    })
  };
  println!("Current OpenGL Version: {:?}", gl_instance.get_string(gl::VERSION));
  let display = unsafe { xlib::XOpenDisplay(ptr::null()) };
  if display == ptr::null_mut() {
    panic!("get display fail");
  }
  let native_display = NativeDisplay::new(display);
  println!("native_display");
  let pdd = native_display.platform_display_data();
  println!("pdd");
  let gl_ctx = SkiaGLContext::new(gl_instance, pdd, size).expect("Fail to create glx_ctx");
  let pixmap = unsafe {
    // Create the pixmap.
    match native_display {
      NativeDisplay::GLX(display) => {
        let screen = xlib::XDefaultScreen(display.display);
        let window = xlib::XRootWindow(display.display, screen);
        // The X server we use for testing on build machines always returns
        // visuals that report 24 bit depth. But creating a 32 bit pixmap does work, so
        // hard code the depth here.
        xlib::XCreatePixmap(display.display,
                            window,
                            size.width as c_uint,
                            size.height as c_uint,
                            24)
      },
    }
  };
  let gl_rasterization_context = GLRasterizationContext::new(gl_ctx, pixmap, size).unwrap();
  DrawTarget::new_with_gl_rasterization_context(Arc::new(gl_rasterization_context), SurfaceFormat::B8G8R8A8)
}
