use pathfinder_font_renderer::{FontContext};

pub fn create_context() -> FontContext {
  FontContext::new().expect("create FontContext fail")
}
