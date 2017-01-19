extern crate byteorder;
#[macro_use]
extern crate lazy_static;

use byteorder::{LittleEndian, ByteOrder};

macro_rules! opt {
    ($e:expr) => (opt_or!($e, None))
}

macro_rules! opt_or {
  ($e:expr, $ret:expr) => {{
    match $e {
      Some(x) => x,
      None => return $ret
    }
  }}
}

#[derive(Debug)]
pub struct RawEntry {
  pub bytes: Vec<u8>
}

impl RawEntry {
  pub fn new(bytes: Vec<u8>) -> Self {
    RawEntry {
      bytes: bytes
    }
  }

  pub fn as_parts(&self) -> Option<RawEntryParts> {
    let header = opt!(self.get_header());
    let second_colon = opt!(self.bytes[9..].iter().position(|b| b == &0x3a));
    let sender = self.bytes[9..second_colon + 9].to_vec();
    let message = self.bytes[second_colon + 9 + 1..].to_vec();
    Some(RawEntryParts {
      header: header,
      sender: sender,
      message: message
    })
  }

  fn get_header(&self) -> Option<Vec<u8>> {
    if self.bytes.len() < 8 {
      return None;
    }
    Some(self.bytes[..8].to_vec())
  }
}

#[derive(Debug)]
pub struct RawEntryParts {
  pub header: Vec<u8>,
  pub sender: Vec<u8>,
  pub message: Vec<u8>
}

impl RawEntryParts {
  pub fn as_entry(&self) -> Entry {
    let entry_type = self.header[4];
    let timestamp = LittleEndian::read_u32(&self.header[..4]);
    let sender = if self.sender.is_empty() {
      None
    } else {
      if let Some(part) = NamePart::parse(&self.sender) {
        Some(part)
      } else if let Ok(name) = String::from_utf8(self.sender.clone()) {
        Some(NamePart::from_names(&name, &name))
      } else if self.sender.len() > 0 {
        Some(Part::Bytes(self.sender.clone()))
      } else {
        None
      }
    };
    let message = Message::new(MessageParser::parse(&self.message));
    Entry {
      entry_type: entry_type,
      timestamp: timestamp,
      sender: sender,
      message: message
    }
  }
}

#[derive(Debug)]
pub struct Entry {
  pub entry_type: u8,
  pub timestamp: u32,
  pub sender: Option<Part>,
  pub message: Message
}

#[derive(Debug)]
pub struct Message {
  pub parts: Vec<Part>
}

impl Message {
  fn new(parts: Vec<Part>) -> Self {
    Message {
      parts: parts
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

#[derive(Debug)]
pub enum Part {
  Name { real_name: Box<Part>, display_name: Box<Part> },
  AutoTranslate { category: u8, id: usize },
  Selectable { info: Vec<u8>, display: Box<Part> },
  Multi(Vec<Box<Part>>),
  PlainText(String),
  Bytes(Vec<u8>)
}

impl HasDisplayText for Part {
  fn display_text(&self) -> String {
    match *self {
      Part::PlainText(ref text) => text.clone(),
      Part::Name { real_name: _, ref display_name } => display_name.display_text(),
      Part::AutoTranslate { category, id } => format!("<AT: {}, {}>", category, id),
      Part::Bytes(ref bytes) => bytes.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" "),
      Part::Selectable { info: _, ref display } => display.display_text(),
      Part::Multi(ref parts) => parts.iter().map(|x| x.display_text()).collect::<Vec<_>>().join("")
    }
  }
}

#[derive(Debug)]
pub struct NamePart;

impl NamePart {
  fn from_names<S>(real_name: S, display_name: S) -> Part
    where S: AsRef<str>
  {
    let real = Part::PlainText(real_name.as_ref().to_owned());
    let disp = Part::PlainText(display_name.as_ref().to_owned());
    Part::Name {
      real_name: Box::new(real),
      display_name: Box::new(disp)
    }
  }

  fn from_parts(real_part: Part, display_part: Part) -> Part {
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
    return true;
  }
}

impl DeterminesLength for NamePart {
  fn determine_length(bytes: &[u8]) -> usize {
    let marker = NamePart::marker_bytes();
    let end_pos = opt_or!(bytes[2..].windows(2).position(|w| w == &[marker.0, marker.1]), 0);
    let last_three = opt_or!(bytes[end_pos + 2..].iter().position(|b| b == &0x03), 0);
    let sum = 2 + end_pos + last_three;
    sum as usize
  }
}

pub fn to_hex_string(bytes: &[u8]) -> String {
  bytes.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ")
}

impl Parses for NamePart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !NamePart::verify_data(bytes) {
      return None;
    }
    let marker = NamePart::marker_bytes();
    let real_length = bytes[2] as usize + 2;
    let display_end = opt!(bytes[real_length..].windows(2).position(|w| w == &[marker.0, marker.1])) + real_length;
    let real_bytes = &bytes[9..real_length];
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

pub struct AutoTranslatePart;

impl AutoTranslatePart {
  fn from_parts(category: u8, id: usize) -> Part {
    Part::AutoTranslate {
      category: category,
      id: id
    }
  }

  fn byte_array_to_be(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 1 {
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
    return true;
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
    let id = opt!(AutoTranslatePart::byte_array_to_be(&bytes[4..2 + length as usize]));
    Some(AutoTranslatePart::from_parts(category, id))
  }
}

struct PlainTextPart;

impl PlainTextPart {
  fn new<S>(text: S) -> Part
    where S: AsRef<str>
  {
    Part::PlainText(text.as_ref().to_owned())
  }
}

struct MultiPart;

impl MultiPart {
  fn from_parts(parts: Vec<Part>) -> Part {
    let boxed_parts = parts.into_iter().map(Box::new).collect();
    Part::Multi(boxed_parts)
  }
}

struct SelectablePart;

impl SelectablePart {
  fn from_parts(info: Vec<u8>, display: Part) -> Part {
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
    return true;
  }
}

impl DeterminesLength for SelectablePart {
  fn determine_length(bytes: &[u8]) -> usize {
    let marker = SelectablePart::marker_bytes();
    let end_pos = opt_or!(bytes[2..].windows(2).rposition(|w| w == &[marker.0, marker.1]), 0);
    let last_three = opt_or!(bytes[end_pos + 2..].iter().position(|b| b == &0x03), 0);
    let sum = 2 + end_pos + last_three;
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
    let mut i = 0;
    while i < message.len() {
      let byte = message[i];
      if byte == 0x02 {
        if let Some((len, part)) = MessageParser::parse_structure(&message[i..]) {
          if !buf.is_empty() {
            parts.push(PlainTextPart::new(String::from_utf8_lossy(&buf)));
            buf.clear();
          }
          parts.push(part);
          i += len + 1;
          continue;
        }
      }
      buf.push(byte);
      i += 1;
    }
    if !buf.is_empty() {
      parts.push(PlainTextPart::new(String::from_utf8_lossy(&buf)));
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
      SelectablePart)
  }
}
