use messages::parts::Part;
use messages::{Parses, DeterminesLength, VerifiesData, HasMarkerBytes};

pub struct PercentagePart;

impl PercentagePart {
  pub fn from_parts(data: u8) -> Part {
    Part::Percentage(data)
  }
}

impl HasMarkerBytes for PercentagePart {
  fn marker_bytes() -> (u8, u8) {
    static MARKER: (u8, u8) = (0x02, 0x1d);
    MARKER
  }
}

impl VerifiesData for PercentagePart {
  fn verify_data(bytes: &[u8]) -> bool {
    if bytes.len() != 4 {
      return false;
    }
    let (two, marker) = PercentagePart::marker_bytes();
    if bytes[0] != two || bytes[1] != marker {
      return false;
    }
    true
  }
}

impl DeterminesLength for PercentagePart {
  fn determine_length(_: &[u8]) -> usize {
    4
  }
}

impl Parses for PercentagePart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !PercentagePart::verify_data(bytes) {
      return None;
    }
    let data = bytes[2];
    Some(PercentagePart::from_parts(data))
  }
}
