extern crate ffxiv_reader;
extern crate serde_json;

use ffxiv_reader::messages::entries::Entry;
use ffxiv_reader::messages::MessageType;
use ffxiv_reader::messages::parts::Part;
use ffxiv_reader::messages::HasDisplayText;
use std::env::args;
use std::fs::File;
use std::io::Read;

fn main() {
  let args: Vec<String> = args().skip(1).collect();
  if args.is_empty() {
    println!("Specify a file with one JSON object per line.");
    return;
  }
  let file_name = &args[0];
  let mut file = match File::open(file_name) {
    Ok(f) => f,
    Err(e) => {
      println!("Could not open {}: {}", file_name, e);
      return;
    }
  };
  let mut data = String::new();
  if let Err(e) = file.read_to_string(&mut data) {
    println!("Could not read {}: {}", file_name, e);
    return;
  }
  let lines = data.split('\n').filter(|x| !x.is_empty() && x.starts_with('{'));
  let entries: Result<Vec<Entry>, serde_json::Error> = lines.map(serde_json::from_str).collect();
  let entries = match entries {
    Ok(e) => e,
    Err(e) => {
      println!("Could not parse JSON as entries: {}", e);
      return;
    }
  };
  for entry in entries {
    if entry.message_type != MessageType::Party &&
       entry.message_type != MessageType::StandardEmotes &&
       entry.message_type != MessageType::CustomEmotes {
      continue;
    }
    let (real, display) = match entry.sender {
      None => continue,
      Some(s) => {
        if let Part::Name { real_name, display_name } = s {
          (real_name.display_text(), display_name.display_text())
        } else if let Part::PlainText(name) = s {
          (name.clone(), name)
        } else {
          continue;
        }
      }
    };
    let stripped_real = strip_party_bytes(&real);
    if stripped_real != "Some Name" && stripped_real != "Other Name" {
      continue;
    }
    println!("({}) {}", display, entry.message.display_text());
  }
}

fn strip_party_bytes(name: &str) -> &str {
  if name.as_bytes()[..2] != [0xEE, 0x82] {
    return name;
  }
  let third = name.as_bytes()[2];
  if third >= 0x90 && third <= 0x97 {
    return unsafe { std::str::from_utf8_unchecked(&name.as_bytes()[3..]) };
  }
  name
}
