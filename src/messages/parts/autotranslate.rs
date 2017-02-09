extern crate flate2;

use self::flate2::read::GzDecoder;

use messages::parts::Part;
use messages::{Parses, DeterminesLength, VerifiesData, HasMarkerBytes};

use std::io::Read;

const DATABASE_JSON_GZ: &'static [u8] = include_bytes!("../../../autotranslate.json.gz");

#[derive(Debug, Deserialize)]
pub struct Completion {
  pub category: u64,
  pub id: u64,
  pub value: String
}

lazy_static! {
  pub static ref DATABASE: Vec<Completion> = {
    let mut reader = GzDecoder::new(DATABASE_JSON_GZ).unwrap();
    let mut data = String::new();
    reader.read_to_string(&mut data).unwrap();
    ::serde_json::from_str(&data).unwrap()
  };
}

pub struct AutoTranslatePart;

impl AutoTranslatePart {
  pub fn from_parts(category: u8, id: usize) -> Part {
    Part::AutoTranslate {
      category: category,
      id: id
    }
  }

  pub fn get_completion(category: u8, id: usize) -> Option<&'static Completion> {
    DATABASE.iter().find(|x| x.category == category as u64 && x.id == id as u64)
  }

  pub fn get_completion_for_part(part: &Part) -> Option<&'static Completion> {
    match *part {
      Part::AutoTranslate { category, id } => AutoTranslatePart::get_completion(category, id),
      _ => None
    }
  }

  fn read_var_be(bytes: &[u8]) -> Option<u64> {
    if bytes.len() == 1 {
      return Some(bytes[0] as u64);
    } else if bytes.is_empty() || bytes.len() > 8 || bytes.len() % 2 == 1 {
      return None;
    }
    let len = bytes.len();
    let mut res: u64 = 0;
    for (i, byte) in bytes.iter().enumerate() {
      res |= (*byte as u64) << (8 * (len - i - 1));
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
    bytes[2] as usize + 3
  }
}

impl Parses for AutoTranslatePart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !AutoTranslatePart::verify_data(bytes) {
      return None;
    }
    let length = bytes[2];
    let category = bytes[3];
    let (skip, adjust) = if length != 3 {
      (5, 0)
    } else {
      (4, 1)
    };
    let mut raw_bytes = Vec::from(&bytes[skip..2 + length as usize]);
    if length % 2 != 1 {
      raw_bytes.insert(0, 0);
    }
    let id = opt!(AutoTranslatePart::read_var_be(&raw_bytes));
    Some(AutoTranslatePart::from_parts(category, id as usize - adjust))
  }
}
