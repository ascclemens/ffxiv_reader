mod name;
mod autotranslate;
mod plaintext;
mod multi;
mod colored;
mod formatted;
mod percentage;
mod icon;

pub use self::name::NamePart;
pub use self::autotranslate::AutoTranslatePart;
pub use self::plaintext::PlainTextPart;
pub use self::multi::MultiPart;
pub use self::colored::ColoredPart;
pub use self::formatted::FormattedPart;
pub use self::percentage::PercentagePart;
pub use self::icon::IconPart;

use messages::HasDisplayText;

/// Parts of a message.
#[derive(Debug, Serialize, Deserialize)]
pub enum Part {
  /// A name, which is composed of a real name and a display name.
  #[serde(rename = "name")]
  Name {
    /// The real name, which is sometimes just information.
    real_name: Box<Part>,
    /// The display name, which is shown to the user.
    display_name: Box<Part>
  },

  /// An auto-translate string.
  ///
  /// The auto-translate database is not yet included.
  #[serde(rename = "auto_translate")]
  AutoTranslate {
    /// The category of the string.
    category: u8,
    /// The id of the string.
    id: usize
  },

  /// A colored part of the message.
  #[serde(rename = "colored")]
  Colored {
    /// The information about the colored part.
    info: Vec<u8>,
    /// The display part for the colored part.
    display: Box<Part>
  },

  /// A part composed of multiple other parts.
  ///
  /// For example, the display name of the `Name` part may be an `Icon` and a `PlainText`.
  #[serde(rename = "multi")]
  Multi(Vec<Box<Part>>),

  /// A plain text part.
  #[serde(rename = "plain_text")]
  PlainText(String),

  /// A part composed of bytes that are unknown.
  ///
  /// If something cannot be parsed currently, it will turn into this variant and contain the raw
  /// bytes that could not be parsed.
  #[serde(rename = "bytes")]
  Bytes(Vec<u8>),

  /// A formatted part.
  ///
  /// Mainly used for italicizing text.
  #[serde(rename = "formatted")]
  Formatted {
    /// The information about the formatted part.
    ///
    /// `[1]` may be the code for italics.
    info: Vec<u8>,
    /// The part to be formatted and displayed.
    display: Box<Part>
  },

  /// Information about a percentage.
  ///
  /// Unsure about what this is really used for. Only seen next to damage numbers with additional
  /// damage (e.g. `(+67%)`) in the battle log.
  #[serde(rename = "percentage")]
  Percentage(u8),

  /// An icon in the text.
  ///
  /// The contained integer is the icon ID, most likely.
  ///
  /// Some icons use this structure, some are UTF-8 glyphs.
  #[serde(rename = "icon")]
  Icon(u64)
}

impl HasDisplayText for Part {
  fn display_text(&self) -> String {
    match *self {
      Part::PlainText(ref text) => text.clone(),
      Part::Name { ref display_name, .. } => display_name.display_text(),
      Part::AutoTranslate { category, id } => {
        match AutoTranslatePart::get_completion(category, id) {
          Some(c) => format!("{{{}}}", c.values.en),
          None => format!("<AT: {}, {}>", category, id)
        }
      },
      Part::Bytes(ref bytes) => bytes.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" "),
      Part::Colored { ref display, .. }
        | Part::Formatted { ref display, .. } => display.display_text(),
      Part::Multi(ref parts) => parts.iter().map(|x| x.display_text()).collect::<Vec<_>>().join(""),
      Part::Percentage(_) => String::from(" "),
      Part::Icon(id) => format!("<Icon: {}>", id)
    }
  }
}
