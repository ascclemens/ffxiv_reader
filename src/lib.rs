extern crate byteorder;
#[macro_use]
extern crate lazy_static;
extern crate memreader;

use byteorder::{LittleEndian, ByteOrder};
use std::sync::mpsc::Receiver;
use memreader::MemReader;

// const CHAT_ADDRESS: usize = 0x2C270010;
// const CHAT_ADDRESS: usize = 0x2CB90010;
const CHAT_ADDRESS: usize = 0x2CA25580;
// const CHAT_ADDRESS: usize = 0x530;
// const CHAT_ADDRESS: usize = 0xFA8;

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
    } else if let Some(part) = NamePart::parse(&self.sender) {
      Some(part)
    } else if let Ok(name) = String::from_utf8(self.sender.clone()) {
      Some(NamePart::from_names(&name, &name))
    } else if !self.sender.is_empty() {
      Some(Part::Bytes(self.sender.clone()))
    } else {
      None
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
      Part::Name { ref display_name, .. } => display_name.display_text(),
      Part::AutoTranslate { category, id } => format!("<AT: {}, {}>", category, id),
      Part::Bytes(ref bytes) => bytes.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" "),
      Part::Selectable { ref display, .. } => display.display_text(),
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
    true
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
    let id = opt!(AutoTranslatePart::byte_array_to_be(&bytes[4..2 + length as usize]));
    Some(AutoTranslatePart::from_parts(category, id))
  }
}

struct PlainTextPart;

impl PlainTextPart {
  fn from_text<S>(text: S) -> Part
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
    true
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
            match String::from_utf8(buf.to_vec()) {
              Ok(s) => parts.push(PlainTextPart::from_text(s)),
              Err(_) => parts.push(Part::Bytes(buf.to_vec()))
            }
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
      SelectablePart)
  }
}

pub struct FfxivMemoryLogReader {
  pid: u32,
  rx: Option<Receiver<Vec<u8>>>
}

impl FfxivMemoryLogReader {
  pub fn new(pid: u32) -> Self {
    FfxivMemoryLogReader {
      pid: pid,
      rx: None
    }
  }

  fn start(&self) -> Option<Receiver<Vec<u8>>> {
    // Create a reader around the PID of the game.
    let reader = match MemReader::new(self.pid) {
      Ok(r) => r,
      Err(e) => {
        println!("Encountered error {} when trying to access memory.", e);
        return None;
      }
    };
    let (tx, rx) = std::sync::mpsc::channel();
    // FIXME: Checking every half second is obviously a race condition. Keep track of last date and
    //        ensure next date is greater than or equal to it. If not, assume incomplete data and
    //        read more.
    std::thread::spawn(move || {
      // Create a buffer for read bytes that aren't full messages
      let mut buffer: Vec<u8> = Vec::new();
      // Keep track of the number of iterations
      let mut iterations = 0;
      // Read the first four bytes (the date of the first message) to check against later
      let mut first_four = reader.read_bytes(CHAT_ADDRESS, 4).unwrap();
      // Bytes to offset when reading after waiting
      let mut offset: isize = 0;
      // Read 32 bytes at a time
      let chunk_size = 32;
      loop {
        // Check to see if the first message's date has changed. If it has, update our stored version
        // and reset the iterations to start reading from the beginning of the memory block. This can
        // totally miss the last message or last couple of messages depending on how frequently this
        // runs. Moving this check to the end of the loop might fix this? TODO
        // May need to clear buffer?
        let check = reader.read_bytes(CHAT_ADDRESS, 4).unwrap();
        if check != first_four {
          first_four = check;
          iterations = 0;
        }
        let unoffset_addr = CHAT_ADDRESS + (iterations * chunk_size);
        let addr: isize = unoffset_addr as isize + offset;
        // Read the next chunk
        let mut read_bytes = reader.read_bytes(addr as usize, chunk_size).unwrap();
        // Create a new vector to contain the buffer plus the newly read bytes
        let mut bytes = Vec::new();
        // Add the buffer
        bytes.append(&mut buffer);
        // Add the just-read bytes
        bytes.append(&mut read_bytes.clone());
        // Increment the iteration counter
        iterations += 1;
        // TODO: Consider making this a while let to clear the buffer before reading more (this may also
        //       involve increasing the chunk size substantially to be more efficient)
        // If we find a null byte outside of the header
        if let Some(i) = bytes[8..].iter().position(|b| b == &0x00) {
          // If there's not enough data to check if we have a full message, add all the data back to the
          // buffer and start again.
          if i + 8 + 8 >= bytes.len() {
            buffer.append(&mut bytes);
            continue;
          }
          // At this point, we know we're somewhere in the header, but not where. To account for the
          // possibility of both null bytes being in the timestamp and colons being in the header, we
          // assume here that the last byte of the header is always 0x00, which is an unsafe assumption,
          // but always seems to be true.
          // Find the rightmost null byte
          let last_null = match bytes[i + 8 .. i + 8 + 8].iter().rposition(|b| b == &0x00) {
            Some(n) => n,
            None => 0 // we're already at the null byte
          };
          // The colon's index will be next to the null byte
          let colon = i + 8 + last_null + 1;
          // If we found a colon at the assumed index
          if bytes[colon] == 0x3a {
            offset = 0;
            // Add the message to the message vector
            tx.send(bytes[..colon - 8].to_vec()).unwrap();
            // messages.push(bytes[..colon - 8].to_vec());
            // Add the rest of the bytes back to the buffer
            buffer.append(&mut bytes[colon - 8..].to_vec());
          // If we didn't find a colon, which is indicative of being at the end of the log
          } else {
            if bytes.iter().any(|x| x != &0x00) {
              let message = &bytes[..colon - 8];
              tx.send(message.to_vec()).unwrap();
              // set offset
              let mut last_byte = 0;
              let mut pos = 0;
              for byte in bytes.iter().rev() {
                if last_byte == 0 && *byte != 0 {
                  break;
                }
                last_byte = *byte;
                pos += 1;
              }
              let new_offset = chunk_size as isize - pos as isize;
              if offset != 0 {
                iterations += 1;
                offset = chunk_size as isize - new_offset + (offset * -1);
                if read_bytes.len() <= chunk_size {
                  offset *= -1;
                }
              } else {
                offset = new_offset;
              }
            } else {
              std::thread::sleep(std::time::Duration::from_millis(500));
            }
            iterations -= 1;
          }
        // If we don't find a null byte outside of the header
        } else {
          // Add all the bytes back to the buffer and start again
          buffer.append(&mut bytes);
        }
      }
    });
    Some(rx)
  }
}

impl Iterator for FfxivMemoryLogReader {
  type Item = Entry;

  fn next(&mut self) -> Option<Entry> {
    if self.rx.is_none() {
      let rx = opt!(self.start());
      self.rx = Some(rx);
    }
    let rx = match self.rx {
      Some(ref r) => r,
      None => return None
    };
    let bytes = rx.recv().unwrap();
    let raw = RawEntry::new(bytes);
    let parts = opt!(raw.as_parts());
    let entry = parts.as_entry();
    Some(entry)
  }
}
