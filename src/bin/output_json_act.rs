extern crate ffxiv_reader;
extern crate serde_json;

use ffxiv_reader::*;

use std::env::args;

// The main loop checks the game's memory for a list of indices that point to where messages start
// in the chat log kept in memory. The loop checks for new indices by checking a pointer, then reads
// any new messages it hasn't read before by reading from the chat log in memory at the index
// locations.

// TODO: Investigate what happens when the memory fills up and starts from the beginning again.
//       Written what I think is a reasonable bit of logic for handling it, but I haven't actually
//       tested it.
// TODO: Handle the game closing, logging out, disconnects, etc. better. Wait for pointers to become
//       valid again, then start reading again.

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
    println!("{}", serde_json::to_string(&entry).unwrap());
  }
}
