use azure::azure_hl::{BackendType, DrawTarget, SurfaceFormat};
use euclid::{Size2D};

pub fn get_draw_target(size: Size2D<i32>) -> DrawTarget {
  DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
}
