use std::fmt::{Debug, Display, Formatter};
use std::fmt::Result as FmtResult;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
  #[serde(rename = "none")]
  None,

  #[serde(rename = "debug")]
  Debug,

  #[serde(rename = "urgent_information")]
  UrgentInformation,

  #[serde(rename = "general_information")]
  GeneralInformation,

  #[serde(rename = "say")]
  Say,

  #[serde(rename = "shout")]
  Shout,

  #[serde(rename = "tell")]
  Tell,

  #[serde(rename = "tell_receive")]
  TellReceive,

  #[serde(rename = "party")]
  Party,

  #[serde(rename = "alliance")]
  Alliance,

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

  #[serde(rename = "free_company")]
  FreeCompany,

  #[serde(rename = "novice_network")]
  NoviceNetwork,

  #[serde(rename = "custom_emotes")]
  CustomEmotes,

  #[serde(rename = "standard_emotes")]
  StandardEmotes,

  #[serde(rename = "yell")]
  Yell,

  #[serde(rename = "party_2")]
  Party2,

  #[serde(rename = "damage")]
  Damage,

  #[serde(rename = "failed_attacks")]
  FailedAttacks,

  #[serde(rename = "actions")]
  Actions,

  #[serde(rename = "items")]
  Items,

  #[serde(rename = "healing_magic")]
  HealingMagic,

  #[serde(rename = "beneficial_effects")]
  BeneficialEffects,

  #[serde(rename = "detrimental_effects")]
  DetrimentalEffects,

  #[serde(rename = "echo")]
  Echo,

  #[serde(rename = "system_messages")]
  SystemMessages,

  #[serde(rename = "system_error_messages")]
  SystemErrorMessages,

  #[serde(rename = "battle_system_messages")]
  BattleSystemMessages,

  #[serde(rename = "gathering_system_messages")]
  GatheringSystemMessages,

  #[serde(rename = "npc_say")]
  NpcSay,

  #[serde(rename = "loot_notices")]
  LootNotices,

  #[serde(rename = "character_progress")]
  CharacterProgress,

  #[serde(rename = "loot_messages")]
  LootMessages,

  #[serde(rename = "crafting_messages")]
  CraftingMessages,

  #[serde(rename = "gathering_messages")]
  GatheringMessages,

  #[serde(rename = "npc_announcements")]
  NpcAnnouncements,

  #[serde(rename = "fc_announcements")]
  FcAnnouncements,

  #[serde(rename = "fc_login_messages")]
  FcLoginMessages,

  #[serde(rename = "retainer_sale_reports")]
  RetainerSaleReports,

  #[serde(rename = "party_search_info")]
  PartySearchInfo,

  #[serde(rename = "sign_settings")]
  SignSettings,

  #[serde(rename = "dice_rolls")]
  DiceRolls,

  #[serde(rename = "music_change")]
  MusicChange,

  #[serde(rename = "novice_network_notifications")]
  NoviceNetworkNotifications,

  #[serde(rename = "gm_tell")]
  GmTell,

  #[serde(rename = "gm_say")]
  GmSay,

  #[serde(rename = "gm_shout")]
  GmShout,

  #[serde(rename = "gm_yell")]
  GmYell,

  #[serde(rename = "gm_party")]
  GmParty,

  #[serde(rename = "gm_free_company")]
  GmFreeCompany,

  #[serde(rename = "gm_linkshell_1")]
  GmLinkshell1,

  #[serde(rename = "gm_linkshell_2")]
  GmLinkshell2,

  #[serde(rename = "gm_linkshell_3")]
  GmLinkshell3,

  #[serde(rename = "gm_linkshell_4")]
  GmLinkshell4,

  #[serde(rename = "gm_linkshell_5")]
  GmLinkshell5,

  #[serde(rename = "gm_linkshell_6")]
  GmLinkshell6,

  #[serde(rename = "gm_linkshell_7")]
  GmLinkshell7,

  #[serde(rename = "gm_linkshell_8")]
  GmLinkshell8,

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

  #[serde(rename = "battle_lose_buff")]
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
      0 => MessageType::None,
      1 => MessageType::Debug,
      2 => MessageType::UrgentInformation,
      3 => MessageType::GeneralInformation,
      10 => MessageType::Say,
      11 => MessageType::Shout,
      12 => MessageType::Tell,
      13 => MessageType::TellReceive,
      14 => MessageType::Party,
      15 => MessageType::Alliance,
      16 => MessageType::Linkshell1,
      17 => MessageType::Linkshell2,
      18 => MessageType::Linkshell3,
      19 => MessageType::Linkshell4,
      20 => MessageType::Linkshell5,
      21 => MessageType::Linkshell6,
      22 => MessageType::Linkshell7,
      23 => MessageType::Linkshell8,
      24 => MessageType::FreeCompany,
      27 => MessageType::NoviceNetwork,
      28 => MessageType::CustomEmotes,
      29 => MessageType::StandardEmotes,
      30 => MessageType::Yell,
      32 => MessageType::Party2,
      41 => MessageType::Damage,
      42 => MessageType::FailedAttacks,
      43 => MessageType::Actions,
      44 => MessageType::Items,
      45 => MessageType::HealingMagic,
      46 => MessageType::BeneficialEffects,
      47 => MessageType::DetrimentalEffects,
      56 => MessageType::Echo,
      57 => MessageType::SystemMessages,
      58 => MessageType::BattleSystemMessages,
      59 => MessageType::GatheringSystemMessages,
      60 => MessageType::SystemErrorMessages,
      61 => MessageType::NpcSay,
      62 => MessageType::LootNotices,
      64 => MessageType::CharacterProgress,
      65 => MessageType::LootMessages,
      66 => MessageType::CraftingMessages,
      67 => MessageType::GatheringMessages,
      68 => MessageType::NpcAnnouncements,
      69 => MessageType::FcAnnouncements,
      70 => MessageType::FcLoginMessages,
      71 => MessageType::RetainerSaleReports,
      72 => MessageType::PartySearchInfo,
      73 => MessageType::SignSettings,
      74 => MessageType::DiceRolls,
      75 => MessageType::NoviceNetworkNotifications,
      76 => MessageType::MusicChange,
      80 => MessageType::GmTell,
      81 => MessageType::GmSay,
      82 => MessageType::GmShout,
      83 => MessageType::GmYell,
      84 => MessageType::GmParty,
      85 => MessageType::GmFreeCompany,
      86 => MessageType::GmLinkshell1,
      87 => MessageType::GmLinkshell2,
      88 => MessageType::GmLinkshell3,
      89 => MessageType::GmLinkshell4,
      90 => MessageType::GmLinkshell5,
      91 => MessageType::GmLinkshell6,
      92 => MessageType::GmLinkshell7,
      93 => MessageType::GmLinkshell8,
      169 => MessageType::BattleReceiveDamage,
      170 => MessageType::BattleResistDebuff,
      171 => MessageType::BattleCast,
      172 => MessageType::ReadyItem,
      174 => MessageType::BattleGainBuff,
      173 => MessageType::BattleSelfAbsorb,
      175 => MessageType::BattleSufferDebuff,
      176 => MessageType::BattleLoseBuff,
      177 => MessageType::BattleRecoverDebuff,
      185 => MessageType::TrialUpdate,
      186 => MessageType::BattleDeath,
      190 => MessageType::GainMgp,
      _ => MessageType::Unknown(u)
    }
  }
}
