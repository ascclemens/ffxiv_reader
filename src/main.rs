extern crate ffxiv_reader;
extern crate memreader;

use ffxiv_reader::*;

use std::env::args;

// The loop reads data from memory until it finds a null byte outside of the first eight bytes it
// has read. It skips eight bytes because the header of the message it is reading is eight bytes.
// Finding a null byte beyond the header means that we have read the header of the next message, and
// we can then do some rudimentary math to determine where the next message's header starts. If we
// determine there is no next message, then we are at the end of the log, and we should continue
// checking the end of the log for messages. We must also check the timestamp of the first message
// in the log memory chunk, since the game will start to write over the memory when it reaches the
// end of the chunk (first writing all of the log into a file on the disk). If the timestamp of the
// first message changes, we should ensure we have read all the messages at the end of the memory
// chunk, and then we should begin reading at the start of the memory chunk again.

// TODO: Figure out how to keep messages from running into each other when the game starts to write
//       at the beginning of the memory chunk again.
// TODO: Ensure the last messages are read when the first timestamp changes.

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
  // Create a log reader.
  let reader = FfxivMemoryLogReader::new(pid);
  // Print out every entry.
  for entry in reader {
    println!("[{}], {}, <{}> {}", entry.timestamp, entry.entry_type, entry.sender.map(|x| x.display_text()).unwrap_or_else(|| String::from("None")), entry.message.display_text());
  }
}
