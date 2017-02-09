use std::fmt::{Debug, Display, Formatter};
use std::fmt::Result as FmtResult;

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

  #[serde(rename = "ready_item")]
  ReadyItem,

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

impl Display for MessageType {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    Debug::fmt(self, f)
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
      0xac => MessageType::ReadyItem,
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
