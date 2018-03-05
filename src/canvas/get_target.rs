#[cfg(target_os="macos")]
pub use super::get_target_cgl::{get_draw_target};
#[cfg(target_os="linux")]
pub use super::get_target_glx::{get_draw_target};
