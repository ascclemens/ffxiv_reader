extern crate ffxiv_reader;
extern crate serde_json;
extern crate time;

use ffxiv_reader::{Entry, HasDisplayText};
use time::Timespec;
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
  let lines = data.split('\n').filter(|x| !x.is_empty());
  let entries: Result<Vec<Entry>, serde_json::Error> = lines.map(serde_json::from_str).collect();
  let entries = match entries {
    Ok(e) => e,
    Err(e) => {
      println!("Could not parse JSON as entries: {}", e);
      return;
    }
  };
  for entry in entries {
    let t = time::at(Timespec::new(entry.timestamp as i64, 0));
    let time_string = t.strftime("%d/%m/%Y %H:%M:%S").unwrap();

    let sender = match entry.sender {
      Some(s) => format!(" <{}>", s.display_text()),
      None => String::new()
    };

    let message = entry.message.display_text().replace('\r', "\n");

    println!("[{}], {},{} {}",
      time_string,
      entry.message_type,
      sender,
      message);
  }
}
