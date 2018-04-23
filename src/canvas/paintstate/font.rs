use csshelper::parse_fonts_style;

#[derive(Debug, Clone)]
pub struct Font {
  pub font_size: f32,
  pub font_style: FontStyle,
  pub font_family: String,
  pub font_variant: FontVariant,
}

impl Font {
  pub fn new(font_rules: &str) -> Font {
    parse_fonts_style(font_rules)
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
