mod canvas_element;
mod canvas_trait;
mod context_2d;
mod paintstate;
mod get_target;
#[cfg(target_os="macos")] mod get_target_cgl;
#[cfg(target_os="linux")] mod get_target_glx;

pub use self::canvas_element::{CanvasElement, CanvasContextType};
pub use self::paintstate::*;
pub use self::canvas_trait::*;
pub use self::context_2d::*;

pub fn create_canvas(width: i32, height: i32, ctx_type: CanvasContextType) -> CanvasElement {
  CanvasElement::new(width, height, ctx_type).unwrap()
}
