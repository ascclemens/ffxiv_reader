//! Message processing

mod types;

pub mod parts;
pub mod parser;
pub mod entries;

pub use self::types::MessageType;
use messages::parts::Part;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
  pub parts: Vec<Part>
}

impl Message {
  pub fn new(parts: Vec<Part>) -> Self {
    Message {
      parts
    }
  }
}

impl HasDisplayText for Message {
  fn display_text(&self) -> String {
    let display_texts: Vec<String> = self.parts.iter().map(|x| x.display_text()).collect();
    display_texts.join("")
  }
}

pub trait HasDisplayText {
  fn display_text(&self) -> String;
}

pub trait DeterminesLength {
  fn determine_length(bytes: &[u8]) -> usize;
}

pub trait VerifiesData {
  fn verify_data(bytes: &[u8]) -> bool;
}

pub trait Parses {
  fn parse(bytes: &[u8]) -> Option<Part>;
}

pub trait HasMarkerBytes {
  fn marker_bytes() -> (u8, u8);
}
