use messages::parts::Part;
use messages::{Parses, DeterminesLength, VerifiesData, HasMarkerBytes};

pub struct AutoTranslatePart;

impl AutoTranslatePart {
  pub fn from_parts(category: u8, id: usize) -> Part {
    Part::AutoTranslate {
      category: category,
      id: id
    }
  }

  fn byte_array_to_be(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() {
      return None;
    }
    if bytes.len() == 1 {
      return Some(bytes[0] as usize);
    }
    let length = bytes.len();
    let mut res: usize = (bytes[0] as usize) << (8 * (length - 1));
    for (i, b) in bytes[1..].iter().enumerate() {
      let bits = 8 * (length - i - 2);
      res |= (*b as usize) << bits
    }
    Some(res)
  }
}

impl HasMarkerBytes for AutoTranslatePart {
  fn marker_bytes() -> (u8, u8) {
    static MARKER: (u8, u8) = (0x02, 0x2e);
    MARKER
  }
}

impl VerifiesData for AutoTranslatePart {
  fn verify_data(bytes: &[u8]) -> bool {
    if bytes.len() < 6 {
      return false;
    }
    let (two, marker) = AutoTranslatePart::marker_bytes();
    if bytes[0] != two || bytes[1] != marker {
      return false;
    }
    true
  }
}

impl DeterminesLength for AutoTranslatePart {
  fn determine_length(bytes: &[u8]) -> usize {
    bytes[2] as usize + 4
  }
}

impl Parses for AutoTranslatePart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !AutoTranslatePart::verify_data(bytes) {
      return None;
    }
    let length = bytes[2];
    let category = bytes[3];
    let id = opt!(AutoTranslatePart::byte_array_to_be(&bytes[4..2 + length as usize]));
    Some(AutoTranslatePart::from_parts(category, id))
  }
}
