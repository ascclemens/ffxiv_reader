extern crate ffxiv_reader;
extern crate time;

use ffxiv_reader::MemoryEntryReader;
use ffxiv_reader::messages::HasDisplayText;

use std::env::args;
use time::Timespec;

fn main() {
  // Gather the arguments supplied to the program.
  let args: Vec<String> = args().skip(1).collect();
  // Ensure they are not empty.
  if args.is_empty() {
    println!("Please supply a PID.");
    return;
  }
  // Attempt to parse a PID from the first arg.
  let pid: u32 = match args[0].parse() {
    Ok(p) => p,
    Err(e) => {
      println!("Invalid PID: {}.", e);
      return;
    }
  };
  // Check whether the program should continue scanning memory or just stop.
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
  let reader = MemoryEntryReader::new(pid, stop);
  // Print out every entry.
  for entry in reader.iter() {
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
