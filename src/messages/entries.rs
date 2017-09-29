//! The structs for entries

use messages::Message;
use messages::types::MessageType;
use messages::parts::{Part, NamePart};
use messages::parser::MessageParser;
use messages::Parses;

use byteorder::{ByteOrder, LittleEndian};

/// A wrapper around the raw bytes of an entry.
#[derive(Debug)]
pub struct RawEntry {
  /// The bytes in the entry.
  pub bytes: Vec<u8>
}

impl RawEntry {
  pub fn new(bytes: Vec<u8>) -> Self {
    RawEntry {
      bytes
    }
  }

  /// Converts the bytes in their raw parts.
  ///
  /// If the bytes are invalid, this will return `None`.
  pub fn as_parts(&self) -> Option<RawEntryParts> {
    let header = opt!(self.get_header());
    let second_colon = opt!(self.bytes[9..].iter().position(|b| b == &0x3a));
    let sender = self.bytes[9..second_colon + 9].to_vec();
    let message = self.bytes[second_colon + 9 + 1..].to_vec();
    Some(RawEntryParts {
      header,
      sender,
      message
    })
  }

  fn get_header(&self) -> Option<Vec<u8>> {
    if self.bytes.len() < 8 {
      return None;
    }
    Some(self.bytes[..8].to_vec())
  }
}

/// The raw parts of an entry.
#[derive(Debug)]
pub struct RawEntryParts {
  /// The bytes for the header of the entry.
  pub header: Vec<u8>,
  /// The bytes for the sender of the entry.
  pub sender: Vec<u8>,
  /// The bytes for the message of the entry.
  pub message: Vec<u8>
}

impl RawEntryParts {
  /// Converts the raw parts into a processed entry.
  pub fn as_entry(&self) -> Entry {
    let message_type = self.header[4];
    let timestamp = LittleEndian::read_u32(&self.header[..4]);
    let sender = if self.sender.is_empty() {
      None
    } else if let Some(part) = NamePart::parse(&self.sender) {
      Some(part)
    } else if let Ok(name) = String::from_utf8(self.sender.clone()) {
      Some(Part::PlainText(name))
    } else if !self.sender.is_empty() {
      Some(Part::Bytes(self.sender.clone()))
    } else {
      None
    };
    let message = Message::new(MessageParser::parse(&self.message));
    Entry {
      message_type: message_type.into(),
      timestamp,
      sender,
      message
    }
  }
}

/// An entry from FFXIV's chat log.
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
  /// The type of message this entry contains.
  pub message_type: MessageType,
  /// The time the entry was created.
  pub timestamp: u32,
  /// The sender of the message, if any.
  pub sender: Option<Part>,
  /// The message of the entry.
  pub message: Message
}
