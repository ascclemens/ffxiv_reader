use messages::parts::Part;
use messages::{Parses, DeterminesLength, VerifiesData, HasMarkerBytes};
use read_var_le;

pub struct IconPart;

impl IconPart {
  pub fn from_parts(data: u64) -> Part {
    Part::Icon(data)
  }
}

impl HasMarkerBytes for IconPart {
  fn marker_bytes() -> (u8, u8) {
    static MARKER: (u8, u8) = (0x02, 0x12);
    MARKER
  }
}

impl VerifiesData for IconPart {
  fn verify_data(bytes: &[u8]) -> bool {
    if bytes.len() < 3 {
      return false;
    }
    let (two, marker) = IconPart::marker_bytes();
    if bytes[0] != two || bytes[1] != marker {
      return false;
    }
    // check for 3 after len
    true
  }
}

impl DeterminesLength for IconPart {
  fn determine_length(bytes: &[u8]) -> usize {
    let len = bytes[2] as usize;
    let last_three = opt_or!(bytes[2 + len..].iter().position(|b| b == &0x03), 0);
    let sum = 3 + len + last_three;
    sum as usize
  }
}

impl Parses for IconPart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !IconPart::verify_data(bytes) {
      return None;
    }
    let len = bytes[2] as usize;
    let last_three = opt!(bytes[2 + len..].iter().position(|b| b == &0x03));
    let raw_data = &bytes[3..3 + len + last_three - 1];
    let data = opt!(read_var_le(raw_data));
    Some(IconPart::from_parts(data))
  }
}
