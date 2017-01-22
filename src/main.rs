extern crate ffxiv_reader;

use ffxiv_reader::*;

use std::env::args;

// The main loop checks the game's memory for a list of indices that point to where messages start
// in the chat log kept in memory. The loop checks for new indices by checking a pointer, then reads
// any new messages it hasn't read before by reading from the chat log in memory at the index
// locations.

// TODO: Investigate what happens when the memory fills up and starts from the beginning again.

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
  let reader = FfxivMemoryLogReader::new(pid, stop);
  // Print out every entry.
  for entry in reader {
    println!("[{}], {}, <{}> {}", entry.timestamp, entry.message_type, entry.sender.map(|x| x.display_text()).unwrap_or_else(|| String::from("None")), entry.message.display_text());
  }
}
