use messages::parts::{Part, MultiPart};
use messages::{Parses, DeterminesLength, VerifiesData, HasMarkerBytes};
use messages::parser::MessageParser;

pub struct SelectablePart;

impl SelectablePart {
  pub fn from_parts(info: Vec<u8>, display: Part) -> Part {
    Part::Selectable {
      info: info,
      display: Box::new(display)
    }
  }
}

impl HasMarkerBytes for SelectablePart {
  fn marker_bytes() -> (u8, u8) {
    static MARKER: (u8, u8) = (0x02, 0x13);
    MARKER
  }
}

impl VerifiesData for SelectablePart {
  fn verify_data(bytes: &[u8]) -> bool {
    if bytes.len() < 7 {
      return false;
    }
    let (two, marker) = SelectablePart::marker_bytes();
    if bytes[0] != two || bytes[1] != marker {
      return false;
    }
    true
  }
}

impl DeterminesLength for SelectablePart {
  fn determine_length(bytes: &[u8]) -> usize {
    let marker = SelectablePart::marker_bytes();
    let end_pos = opt_or!(bytes[2..].windows(2).rposition(|w| w == &[marker.0, marker.1]), 0);
    let last_three = opt_or!(bytes[end_pos + 2..].iter().position(|b| b == &0x03), 0);
    let sum = 3 + end_pos + last_three;
    sum as usize
  }
}

impl Parses for SelectablePart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !SelectablePart::verify_data(bytes) {
      return None;
    }
    let marker = SelectablePart::marker_bytes();
    let info_length = bytes[2] as usize + 2;
    // lol rposition because you can embed parts inside of parts, and I don't want to do a ton of
    // logic to find out the length properly.
    let display_end = opt!(bytes[info_length..].windows(2).rposition(|w| w == &[marker.0, marker.1])) + info_length;
    let info_bytes = &bytes[3..info_length];
    let display_bytes = &bytes[info_length + 1 .. display_end];
    let mut parts = MessageParser::parse(display_bytes);
    let display_part = if parts.len() == 1 {
      parts.remove(0)
    } else if parts.len() > 1 {
      MultiPart::from_parts(parts)
    } else if let Ok(s) = String::from_utf8(display_bytes.to_vec()) {
      Part::PlainText(s)
    } else {
      Part::Bytes(display_bytes.to_vec())
    };
    Some(SelectablePart::from_parts(info_bytes.to_vec(), display_part))
  }
}
