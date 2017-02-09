use messages::parts::{Part, MultiPart};
use messages::{Parses, DeterminesLength, VerifiesData, HasMarkerBytes};
use messages::parser::MessageParser;

pub struct NamePart;

impl NamePart {
  pub fn from_names<S>(real_name: S, display_name: S) -> Part
    where S: AsRef<str>
  {
    let real = Part::PlainText(real_name.as_ref().to_owned());
    let disp = Part::PlainText(display_name.as_ref().to_owned());
    Part::Name {
      real_name: Box::new(real),
      display_name: Box::new(disp)
    }
  }

  pub fn from_parts(real_part: Part, display_part: Part) -> Part {
    Part::Name {
      real_name: Box::new(real_part),
      display_name: Box::new(display_part)
    }
  }
}

impl HasMarkerBytes for NamePart {
  fn marker_bytes() -> (u8, u8) {
    static MARKER: (u8, u8) = (0x02, 0x27);
    MARKER
  }
}

impl VerifiesData for NamePart {
  fn verify_data(bytes: &[u8]) -> bool {
    if bytes.len() < 22 {
      return false;
    }
    let (two, marker) = NamePart::marker_bytes();
    if bytes[0] != two || bytes[1] != marker {
      return false;
    }
    true
  }
}

impl DeterminesLength for NamePart {
  fn determine_length(bytes: &[u8]) -> usize {
    let marker = NamePart::marker_bytes();
    let end_pos = opt_or!(bytes[2..].windows(2).position(|w| w == &[marker.0, marker.1]), 0);
    let last_three = opt_or!(bytes[end_pos + 2..].iter().position(|b| b == &0x03), 0);
    let sum = 3 + end_pos + last_three;
    sum as usize
  }
}

impl Parses for NamePart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !NamePart::verify_data(bytes) {
      return None;
    }
    let marker = NamePart::marker_bytes();
    let real_length = bytes[2] as usize + 2;
    let display_end = opt!(bytes[real_length..].windows(2).position(|w| w == &[marker.0, marker.1])) + real_length;
    let skip = if bytes[3] == 0x03 {
      3
    } else {
      9
    };
    let real_bytes = &bytes[skip..real_length];
    let real_name = match String::from_utf8(real_bytes.to_vec()) {
      Ok(r) => Part::PlainText(r),
      Err(_) => Part::Bytes(real_bytes.to_vec())
    };
    let display_bytes = &bytes[real_length + 1 .. display_end];
    let mut parts = MessageParser::parse(display_bytes);
    let display_name = if parts.len() == 1 {
      parts.remove(0)
    } else if parts.len() > 1 {
      MultiPart::from_parts(parts)
    } else if let Ok(s) = String::from_utf8(display_bytes.to_vec()) {
      Part::PlainText(s)
    } else {
      Part::Bytes(display_bytes.to_vec())
    };
    Some(NamePart::from_parts(real_name, display_name))
  }
}
