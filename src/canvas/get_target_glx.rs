use azure::azure_hl::{BackendType, DrawTarget, SurfaceFormat};

pub fn get_draw_target(size: Size2D<i32>) -> DrawTarget {
  DrawTraget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
}
