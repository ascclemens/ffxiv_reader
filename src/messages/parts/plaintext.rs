use messages::parts::Part;

pub struct PlainTextPart;

impl PlainTextPart {
  pub fn from_text<S>(text: S) -> Part
    where S: AsRef<str>
  {
    Part::PlainText(text.as_ref().to_owned())
  }
}
