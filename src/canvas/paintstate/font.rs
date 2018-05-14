use std::str::FromStr;

use csshelper::parse_fonts_style;

#[derive(Debug, Clone)]
pub struct Font {
  pub font_size: f32,
  pub font_style: FontStyle,
  pub font_family: String,
  pub font_variant: FontVariant,
}

impl FromStr for Font {
  type Err = ();

  fn from_str(rules: &str) -> Result<Font, ()> {
    Ok(parse_fonts_style(rules))
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
  Normal,
  Italic,
  Oblique,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontVariant {
  Normal,
  SmallCaps,
}

#[cfg(test)]
mod font_test {
  use super::Font;

  #[test]
  fn font_default_check() {
    Font::new("10px sans-serif");
  }
}
