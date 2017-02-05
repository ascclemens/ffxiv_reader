extern crate byteorder;
extern crate memreader;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use byteorder::{LittleEndian, ByteOrder};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use memreader::MemReader;

const CHAT_POINTER: usize = 0x2CA90E58;
const INDEX_POINTER: usize = CHAT_POINTER - 12;
const LINES_ADDRESS: usize = CHAT_POINTER - 52;

// TODO: Handle the game closing, logging out, disconnects, etc. better. Wait for pointers to become
//       valid again, then start reading again.

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
      timestamp: timestamp,
      sender: sender,
      message: message
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
  pub message_type: MessageType,
  pub timestamp: u32,
  pub sender: Option<Part>,
  pub message: Message
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Part {
  #[serde(rename = "name")]
  Name { real_name: Box<Part>, display_name: Box<Part> },

  #[serde(rename = "auto_translate")]
  AutoTranslate { category: u8, id: usize },

  #[serde(rename = "selectable")]
  Selectable { info: Vec<u8>, display: Box<Part> }, // this may be Colored?

  #[serde(rename = "multi")]
  Multi(Vec<Box<Part>>),

  #[serde(rename = "plain_text")]
  PlainText(String),

  #[serde(rename = "bytes")]
  Bytes(Vec<u8>),

  #[serde(rename = "formatted")]
  Formatted { info: Vec<u8>, display: Box<Part> },

  #[serde(rename = "percentage")]
  Percentage(u8),

  #[serde(rename = "icon")]
  Icon(u64)
}

impl HasDisplayText for Part {
  fn display_text(&self) -> String {
    match *self {
      Part::PlainText(ref text) => text.clone(),
      Part::Name { ref display_name, .. } => display_name.display_text(),
      Part::AutoTranslate { category, id } => format!("<AT: {}, {}>", category, id),
      Part::Bytes(ref bytes) => bytes.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" "),
      Part::Selectable { ref display, .. }
        | Part::Formatted { ref display, .. } => display.display_text(),
      Part::Multi(ref parts) => parts.iter().map(|x| x.display_text()).collect::<Vec<_>>().join(""),
      Part::Percentage(_) => String::from(" "),
      Part::Icon(id) => format!("<Icon: {}>", id)
    }
  }
}

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
  pub fn from_parts(parts: Vec<Part>) -> Part {
    let boxed_parts = parts.into_iter().map(Box::new).collect();
    Part::Multi(boxed_parts)
  }
}

struct SelectablePart;

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

struct FormattedPart;

impl FormattedPart {
  pub fn from_parts(info: Vec<u8>, display: Part) -> Part {
    Part::Formatted {
      info: info,
      display: Box::new(display)
    }
  }
}

impl HasMarkerBytes for FormattedPart {
  fn marker_bytes() -> (u8, u8) {
    static MARKER: (u8, u8) = (0x02, 0x1a);
    MARKER
  }
}

impl VerifiesData for FormattedPart {
  fn verify_data(bytes: &[u8]) -> bool {
    if bytes.len() < 7 {
      return false;
    }
    let (two, marker) = FormattedPart::marker_bytes();
    if bytes[0] != two || bytes[1] != marker {
      return false;
    }
    true
  }
}

impl DeterminesLength for FormattedPart {
  fn determine_length(bytes: &[u8]) -> usize {
    let marker = FormattedPart::marker_bytes();
    let end_pos = opt_or!(bytes[2..].windows(2).rposition(|w| w == &[marker.0, marker.1]), 0);
    let last_three = opt_or!(bytes[end_pos + 2..].iter().position(|b| b == &0x03), 0);
    let sum = 3 + end_pos + last_three;
    sum as usize
  }
}

impl Parses for FormattedPart {
  fn parse(bytes: &[u8]) -> Option<Part> {
    if !FormattedPart::verify_data(bytes) {
      return None;
    }
    let marker = FormattedPart::marker_bytes();
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
    Some(FormattedPart::from_parts(info_bytes.to_vec(), display_part))
  }
}

struct PercentagePart;

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

struct IconPart;

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

fn read_var_le(bytes: &[u8]) -> Option<u64> {
  if bytes.len() == 1 {
    return Some(bytes[0] as u64);
  } else if bytes.is_empty() || bytes.len() > 8 || bytes.len() % 2 == 1 {
    return None;
  }
  let mut res: u64 = 0;
  for (i, byte) in bytes.iter().enumerate() {
    res |= (*byte as u64) << (8 * i);
  }
  Some(res)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
  #[serde(rename = "system_message")]
  SystemMessage,

  #[serde(rename = "say")]
  Say,

  #[serde(rename = "shout")]
  Shout,

  #[serde(rename = "reply")]
  Reply,

  #[serde(rename = "tell")]
  Tell,

  #[serde(rename = "party")]
  Party,

  #[serde(rename = "linkshell_1")]
  Linkshell1,

  #[serde(rename = "linkshell_2")]
  Linkshell2,

  #[serde(rename = "linkshell_3")]
  Linkshell3,

  #[serde(rename = "linkshell_4")]
  Linkshell4,

  #[serde(rename = "linkshell_5")]
  Linkshell5,

  #[serde(rename = "linkshell_6")]
  Linkshell6,

  #[serde(rename = "linkshell_7")]
  Linkshell7,

  #[serde(rename = "linkshell_8")]
  Linkshell8,

  #[serde(rename = "free_company_chat")]
  FreeCompanyChat,

  #[serde(rename = "custom_emote")]
  CustomEmote,

  #[serde(rename = "emote")]
  Emote,

  #[serde(rename = "yell")]
  Yell,

  #[serde(rename = "battle_damage")]
  BattleDamage,

  #[serde(rename = "battle_miss")]
  BattleMiss,

  #[serde(rename = "battle_use_action")]
  BattleUseAction,

  #[serde(rename = "use_item")]
  UseItem,

  #[serde(rename = "battle_other_absorb")]
  BattleOtherAbsorb,

  #[serde(rename = "battle_gain_status_effect")]
  BattleGainStatusEffect,

  #[serde(rename = "battle_gain_debuff")]
  BattleGainDebuff,

  #[serde(rename = "item_obtained")]
  ItemObtained,

  #[serde(rename = "client_echo")]
  ClientEcho,

  #[serde(rename = "server_echo")]
  ServerEcho,

  #[serde(rename = "battle_death_revive")]
  BattleDeathRevive,

  #[serde(rename = "error")]
  Error,

  #[serde(rename = "receive_reward")]
  ReceiveReward,

  #[serde(rename = "gain_experience")]
  GainExperience,

  #[serde(rename = "loot")]
  Loot,

  #[serde(rename = "crafting")]
  Crafting,

  #[serde(rename = "npc_chat")]
  NpcChat,

  #[serde(rename = "free_company_event")]
  FreeCompanyEvent,

  #[serde(rename = "log_in_out")]
  LogInOut,

  #[serde(rename = "market_board")]
  MarketBoard,

  #[serde(rename = "party_finder_update")]
  PartyFinderUpdate,

  #[serde(rename = "party_mark")]
  PartyMark,

  #[serde(rename = "random")]
  Random,

  #[serde(rename = "music_change")]
  MusicChange,

  #[serde(rename = "battle_receive_damage")]
  BattleReceiveDamage,

  #[serde(rename = "battle_resist_debuff")]
  BattleResistDebuff,

  #[serde(rename = "battle_cast")]
  BattleCast,

  #[serde(rename = "battle_gain_buff")]
  BattleGainBuff,

  #[serde(rename = "battle_self_absorb")]
  BattleSelfAbsorb,

  #[serde(rename = "battle_suffer_debuff")]
  BattleSufferDebuff,

  #[serde(rename = "battle_lose_debuff")]
  BattleLoseBuff,

  #[serde(rename = "battle_recover_debuff")]
  BattleRecoverDebuff,

  #[serde(rename = "trial_update")]
  TrialUpdate,

  #[serde(rename = "battle_death")]
  BattleDeath,

  #[serde(rename = "gain_mgp")]
  GainMgp,

  #[serde(rename = "unknown")]
  Unknown(u8)
}

impl std::fmt::Display for MessageType {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
  }
}

impl From<u8> for MessageType {
  fn from(u: u8) -> MessageType {
    match u {
      0x03 => MessageType::SystemMessage,
      0x0a => MessageType::Say,
      0x0b => MessageType::Shout,
      0x0c => MessageType::Reply,
      0x0d => MessageType::Tell,
      0x0e => MessageType::Party,
      0x10 => MessageType::Linkshell1,
      0x11 => MessageType::Linkshell2,
      0x12 => MessageType::Linkshell3,
      0x13 => MessageType::Linkshell4,
      0x14 => MessageType::Linkshell5,
      0x15 => MessageType::Linkshell6,
      0x16 => MessageType::Linkshell7,
      0x17 => MessageType::Linkshell8,
      0x18 => MessageType::FreeCompanyChat,
      0x1c => MessageType::CustomEmote,
      0x1d => MessageType::Emote,
      0x1e => MessageType::Yell,
      0x29 => MessageType::BattleDamage,
      0x2a => MessageType::BattleMiss,
      0x2b => MessageType::BattleUseAction,
      0x2c => MessageType::UseItem,
      0x2d => MessageType::BattleOtherAbsorb,
      0x2e => MessageType::BattleGainStatusEffect,
      0x2f => MessageType::BattleGainDebuff,
      0x3e => MessageType::ItemObtained,
      0x38 => MessageType::ClientEcho,
      0x39 => MessageType::ServerEcho,
      0x3a => MessageType::BattleDeathRevive,
      0x3c => MessageType::Error,
      // 0x3e => MessageType::ReceiveReward,
      0x40 => MessageType::GainExperience,
      0x41 => MessageType::Loot,
      0x42 => MessageType::Crafting,
      0x44 => MessageType::NpcChat,
      0x45 => MessageType::FreeCompanyEvent,
      0x46 => MessageType::LogInOut,
      0x47 => MessageType::MarketBoard,
      0x48 => MessageType::PartyFinderUpdate,
      0x49 => MessageType::PartyMark,
      0x4a => MessageType::Random,
      0x4c => MessageType::MusicChange,
      0xa9 => MessageType::BattleReceiveDamage,
      0xaa => MessageType::BattleResistDebuff,
      0xab => MessageType::BattleCast,
      0xae => MessageType::BattleGainBuff,
      0xad => MessageType::BattleSelfAbsorb,
      0xaf => MessageType::BattleSufferDebuff,
      0xb0 => MessageType::BattleLoseBuff,
      0xb1 => MessageType::BattleRecoverDebuff,
      0xb9 => MessageType::TrialUpdate,
      0xba => MessageType::BattleDeath,
      0xbe => MessageType::GainMgp,
      _ => MessageType::Unknown(u)
    }
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
    // FIXME: enumerate
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
          i += len;
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
      SelectablePart,
      FormattedPart,
      PercentagePart,
      IconPart)
  }
}

pub struct FfxivMemoryLogReader {
  pid: u32,
  stop: bool,
  rx: Option<Receiver<Vec<u8>>>,
  run: Arc<AtomicBool>
}

impl FfxivMemoryLogReader {
  pub fn new(pid: u32, stop: bool) -> Self {
    FfxivMemoryLogReader {
      pid: pid,
      stop: stop,
      rx: None,
      run: Arc::new(AtomicBool::new(false))
    }
  }

  pub fn start(&self) -> Option<Receiver<Vec<u8>>> {
    // Create a reader around the PID of the game.
    let reader = match MemReader::new(self.pid) {
      Ok(r) => r,
      Err(e) => {
        println!("Encountered error {} when trying to access memory.", e);
        return None;
      }
    };
    let raw_chat_pointer = reader.read_bytes(CHAT_POINTER, 4).unwrap();
    let chat_address = LittleEndian::read_u32(&raw_chat_pointer) as usize;
    let stop = self.stop;
    let (tx, rx) = std::sync::mpsc::channel();
    self.run.store(true, Ordering::Relaxed);
    let run = self.run.clone();
    std::thread::spawn(move || {
      // Index of last read index
      let mut index_index = 0;
      while run.load(Ordering::Relaxed) {
        // Get raw bytes for current index pointer
        let raw_pointer = reader.read_bytes(INDEX_POINTER, 4).unwrap();
        // Read the raw bytes into an address
        let pointer = LittleEndian::read_u32(&raw_pointer);
        // Read the total number of lines (modulo 1000 because the game wraps around at 1000)
        let num_lines = {
          let raw = reader.read_bytes(LINES_ADDRESS, 4).unwrap();
          LittleEndian::read_u32(&raw) % 1000
        };
        // Read u32s backwards until we hit 0
        let mut mem_indices = Vec::with_capacity(index_index + 1);
        loop {
          // If the amount of lines we've read is equal to the number of lines, break
          if mem_indices.len() == num_lines as usize {
            break;
          }
          // Read backwards, incrementing by four for each index read
          let raw_index = reader.read_bytes(pointer as usize - (4 * (mem_indices.len() + 1)), 4).unwrap();
          // Read the raw bytes into a u32
          let index = LittleEndian::read_u32(&raw_index);
          // Otherwise, insert the index at the start
          mem_indices.insert(0, index);
        }
        // If the number of indices we just read is equal to the last index of the indices we read,
        // there are no new messages, so sleep and restart the loop.
        if mem_indices.len() == index_index {
          if stop {
            break;
          } else {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
          }
        } else if mem_indices.len() < index_index {
          // If the amount of indices we've read is less than the amount we were at last time,
          // we've wrapped around in the memory, so reset the index to 0.
          index_index = 0;
        }
        // Get all the new indices
        let new_indices = &mem_indices[index_index..];
        // Get the last index, or 0 to start
        let mut last_index = if index_index == 0 {
          0
        } else {
          // The last index will be in the new indices we just read, being the last one we have read
          mem_indices[index_index - 1]
        };
        index_index = mem_indices.len();
        // Read each new message and send it
        for index in new_indices {
          let message = reader.read_bytes(chat_address + last_index as usize, *index as usize - last_index as usize).unwrap();
          last_index = *index;
          tx.send(message).unwrap();
        }
      }
    });
    Some(rx)
  }

  pub fn stop(&self) {
    self.run.store(false, Ordering::Relaxed);
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
    let bytes = match rx.recv() {
      Ok(b) => b,
      Err(_) => return None
    };
    let raw = RawEntry::new(bytes);
    let parts = opt!(raw.as_parts());
    let entry = parts.as_entry();
    Some(entry)
  }
}
