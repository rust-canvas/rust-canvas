use cssparser::{Parser, ParserInput, Token};
use std::ops::{Deref};

use super::canvas::{Font, FontStyle, FontVariant};

#[cfg(target_os = "linux")]
pub static SANS_SERIF_FONT_FAMILY: &'static str = "DejaVu Sans";
#[cfg(target_os = "macos")]
pub static SANS_SERIF_FONT_FAMILY: &'static str = "Helvetica";
#[cfg(target_os = "windows")]
pub static SANS_SERIF_FONT_FAMILY: &'static str = "Arial";
pub static SERIF_FONT_FAMILY: &'static str = "Times New Roman";
pub static CURSIVE_FONT_FAMILY: &'static str = "Apple Chancery";
pub static FANTASY_FONT_FAMILY: &'static str = "Papyrus";
pub static MONOSPACE_FONT_FAMILY: &'static str = "Menlo";

pub fn parse_fonts_style(input: &str) -> Font {
  let parser_input = &mut ParserInput::new(input);
  let mut css_parser = Parser::new(parser_input);
  let mut font_size = 10.0;
  let mut font_family = "san-serif".to_string();
  let mut index = 0;
  let mut font_variant = FontVariant::Normal;
  let mut font = None;
  let mut font_style: FontStyle = FontStyle::Normal;
  while !css_parser.is_exhausted() {
    match css_parser.next() {
      Ok(t) => {
        match t {
          &Token::Dimension { ref value, ref unit, .. } => {
            let s = unit.deref().to_lowercase();
            // handle others absolute unit here
            if s == "px" {
              font_size = * value;
            } else if s == "em" {
              font_size = value * (16 as f32);
            };
          },
          &Token::QuotedString(ref d) => {
            font_family = String::from(d.deref());
          },
          &Token::Ident(ref d) => {
            let val = d.deref().to_lowercase();
            if index == 0 || index == 1 {
              if val == "small-caps" {
                font_variant = FontVariant::SmallCaps;
              }
              if index == 0 {
                if val == "italic" {
                  font_style = FontStyle::Italic;
                } else if val == "oblique" {
                  font_style = FontStyle::Oblique;
                } else {
                  println!("font style: {} is illegal", val);
                }
              }
            } else {
              font = Some(String::from(val));
            }
          },
          _ => println!("other branch: {:?}", t),
        };
        index = index + 1;

      },
      Err(_) => {
        font_family = font.unwrap();
        font = None;
      },
    };
  };

  match font {
    Some(s) => font_family = String::from(s),
    None => { },
  };

  let font_family = match font_family.as_str() {
    "serif" => SERIF_FONT_FAMILY,
    "sans-serif" => SANS_SERIF_FONT_FAMILY,
    "cursive" => CURSIVE_FONT_FAMILY,
    "fantasy" => FANTASY_FONT_FAMILY,
    "monospace" => MONOSPACE_FONT_FAMILY,
    other => other,
  };

  Font {
    font_size,
    font_style,
    font_family: font_family.to_string(),
    font_variant,
  }
}

#[cfg(test)]
mod parse_fonts_style_tests {
  use super::*;
  #[test]
  fn should_parse_size_family() {
    let result = parse_fonts_style("2em \"Open Sans\"");
    assert_eq!(result.font_family, String::from("Open Sans"));
    assert_eq!(result.font_size, 32 as f32);
    assert_eq!(result.font_variant, FontVariant::Normal);
  }

  #[test]
  fn should_parse_style_size_family() {
    let result = parse_fonts_style("italic 2em \"Open Sans\"");
    assert_eq!(result.font_family, String::from("Open Sans"));
    assert_eq!(result.font_size, 32 as f32);
    assert_eq!(result.font_style, FontStyle::Italic);
    assert_eq!(result.font_variant, FontVariant::Normal);
  }

  #[test]
  fn should_parse_style_variant_weight_size_lineheight_family() {
    let result = parse_fonts_style("italic small-caps bolder 16px/3 cursive");
    assert_eq!(result.font_family, String::from(CURSIVE_FONT_FAMILY));
    assert_eq!(result.font_size, 16 as f32);
    assert_eq!(result.font_variant, FontVariant::SmallCaps);
  }
}
