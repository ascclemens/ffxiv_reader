use messages::parts::{Part,
  NamePart,
  AutoTranslatePart,
  SelectablePart,
  FormattedPart,
  PercentagePart,
  IconPart,
  PlainTextPart};
use messages::{Parses, DeterminesLength, HasMarkerBytes};

macro_rules! parse_structure_macro {
  ($t:ident, $message:expr) => {{
    let length = $t::determine_length(&$message);
    let part = opt!($t::parse(&$message[..length]));
    Some((length, part))
  }};
}

macro_rules! parse_structure_if_macro {
  ($id:expr, $message:expr, $first_t:ident, $($t:ident),*) => {{
    if $id == $first_t::marker_bytes().1 {
      parse_structure_macro!($first_t, $message)
    }
    $(else if $id == $t::marker_bytes().1 {
      parse_structure_macro!($t, $message)
    })*
    else {
      None
    }
  }};
}

pub struct MessageParser;

impl MessageParser {
  pub fn parse(message: &[u8]) -> Vec<Part> {
    let mut parts: Vec<Part> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    // FIXME: enumerate
    let mut i = 0;
    while i < message.len() {
      let byte = message[i];
      if byte == 0x02 {
        if let Some((len, part)) = MessageParser::parse_structure(&message[i..]) {
          if !buf.is_empty() {
            match String::from_utf8(buf.to_vec()) {
              Ok(s) => parts.push(PlainTextPart::from_text(s)),
              Err(_) => parts.push(Part::Bytes(buf.to_vec()))
            }
            buf.clear();
          }
          parts.push(part);
          i += len;
          continue;
        }
      }
      buf.push(byte);
      i += 1;
    }
    if !buf.is_empty() {
      match String::from_utf8(buf.to_vec()) {
        Ok(s) => parts.push(PlainTextPart::from_text(s)),
        Err(_) => parts.push(Part::Bytes(buf))
      }
    }
    parts
  }

  fn parse_structure(message: &[u8]) -> Option<(usize, Part)> {
    if message.len() < 2 {
      return None;
    }
    let structure_id = message[1];
    parse_structure_if_macro!(
      structure_id,
      message,
      NamePart,
      AutoTranslatePart,
      SelectablePart,
      FormattedPart,
      PercentagePart,
      IconPart)
  }
}
