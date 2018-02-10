use cssparser::{Color};

#[derive(Debug)]
pub enum FillStyle {
  Color(Color),
  Gradient(Gradient),
  Pattern(Pattern),
}

#[derive(Debug)]
pub struct Gradient { }

#[derive(Debug)]
pub struct Pattern { }
