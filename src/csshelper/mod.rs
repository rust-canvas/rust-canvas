use cssparser::{Parser, ParserInput, Token};
use std::ops::{Deref};

use super::canvas::{Font, FontStyle, FontVariant};

pub fn parse_fonts_style(input: &str) -> Font {
  let parser_input = &mut ParserInput::new(input);
  let mut css_parser = Parser::new(parser_input);
  let mut font_size = 10 as f32;
  let mut font_familys = Vec::new();
  let mut index = 0;
  let mut font_variant = FontVariant::Normal;
  let mut font = None;
  while !css_parser.is_exhausted() {
    match css_parser.next() {
      Ok(t) => {
        match t {
          & Token::Dimension { ref value, ref unit, .. } => {
            let s = unit.deref();
            // handle others absolute unit here
            if s == "px" {
              font_size = * value;
            } else if s == "em" {
              font_size = value * (16 as f32);
            };
          },
          & Token::QuotedString(ref d) => {
            font_familys.push(String::from(d.deref()));
          },
          & Token::Ident(ref d) => {
            let val = d.deref();
            if index == 0 || index == 1 {
              if val == "small-caps" {
                font_variant = FontVariant::SmallCaps;
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
        font_familys.push(font.unwrap());
        font = None;
      },
    };
  };

  match font {
    Some(s) => font_familys.push(String::from(s)),
    None => { },
  };

  Font {
    font_size,
    font_style: FontStyle::Normal,
    font_familys, font_variant,
  }
}

#[cfg(test)]
mod parse_fonts_style_tests {
  use super::{parse_fonts_style};
  #[test]
  fn should_parse_size_family() {
    let result = parse_fonts_style("2em \"Open Sans\"");
    assert_eq!(result.font_familys, vec![String::from("Open Sans")]);
    assert_eq!(result.font_size, 32 as f32);
  }

  #[test]
  fn should_parse_style_size_family() {
    let result = parse_fonts_style("italic 2em \"Open Sans\"");
    assert_eq!(result.font_familys, vec![String::from("Open Sans")]);
    assert_eq!(result.font_size, 32 as f32);
  }

  #[test]
  fn should_parse_style_variant_weight_size_lineheight_family() {
    let result = parse_fonts_style("italic small-caps bolder 16px/3 cursive");
    assert_eq!(result.font_familys, vec![String::from("cursive")]);
    assert_eq!(result.font_size, 16 as f32);
  }
}
