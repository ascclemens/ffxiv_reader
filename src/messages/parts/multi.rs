use messages::parts::Part;

pub struct MultiPart;

impl MultiPart {
  pub fn from_parts(parts: Vec<Part>) -> Part {
    let boxed_parts = parts.into_iter().map(Box::new).collect();
    Part::Multi(boxed_parts)
  }
}
