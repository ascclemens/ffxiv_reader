extern crate ffxiv_reader;
extern crate chrono;

use ffxiv_reader::ActReader;
use ffxiv_reader::messages::HasDisplayText;

use std::env::args;
use chrono::{Utc, TimeZone};

fn main() {
  // Gather the arguments supplied to the program.
  let args: Vec<String> = args().skip(1).collect();
  // Ensure they are not empty.
  if args.is_empty() {
    println!("Please supply a path.");
    return;
  }
  // Get path to the file.
  let path = &args[0];
  // Check whether the program should continue scanning the file or just stop.
  let stop = if args.len() > 1 {
    match args[1].to_lowercase().parse() {
      Ok(b) => b,
      Err(e) => {
        println!("Invalid stop argument. Please specify true/false. {}", e);
        return;
      }
    }
  } else { false };
  // Create a log reader.
  let reader = ActReader::new(path, stop);
  let rx = reader.start().unwrap();
  // Print out every entry.
  loop {
    let entry = rx.recv().unwrap();

    let timestamp = Utc.timestamp(entry.timestamp as i64, 0);
    let time_string = timestamp.format("%d/%m/%Y %H:%M:%S");

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
