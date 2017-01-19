extern crate ffxiv_reader;
extern crate memreader;

use ffxiv_reader::*;
use memreader::MemReader;
// use memreader::FileReader;

use std::env::args;

const CHAT_ADDRESS: usize = 0x2C270010;
// const CHAT_ADDRESS: usize = 0x00;

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
  // Create a reader around the PID of the game.
  let reader = match MemReader::new(pid) {
    Ok(r) => r,
    Err(e) => {
      println!("Encountered error {} when trying to access memory.", e);
      return;
    }
  };
  // let mut reader = FileReader::new(&args[0]);
  // Create a buffer for read bytes that aren't full messages
  let mut buffer: Vec<u8> = Vec::new();
  // Create a vector of message bytes
  let mut messages: Vec<Vec<u8>> = Vec::new();
  // Keep track of the number of iterations
  let mut iterations = 0;
  // Read the first four bytes (the date of the first message) to check against later
  let mut first_four = reader.read_bytes(CHAT_ADDRESS, 4).unwrap();
  // Read 32 bytes at a time
  let chunk_size = 32;
  loop {
    // Check to see if the first message's date has changed. If it has, update our stored version
    // and reset the iterations to start reading from the beginning of the memory block. This can
    // totally miss the last message or last couple of messages depending on how frequently this
    // runs. Moving this check to the end of the loop might fix this? TODO
    // May need to clear buffer?
    let check = reader.read_bytes(CHAT_ADDRESS, 4).unwrap();
    if check != first_four {
      first_four = check;
      iterations = 0;
    }
    // Read the next chunk
    let mut read_bytes = reader.read_bytes(CHAT_ADDRESS + (iterations * chunk_size), chunk_size).unwrap();
    // Create a new vector to contain the buffer plus the newly read bytes
    let mut bytes = Vec::new();
    // Add the buffer
    bytes.append(&mut buffer);
    // Add the just-read bytes
    bytes.append(&mut read_bytes);
    // Increment the iteration counter
    iterations += 1;
    // TODO: Consider making this a while let to clear the buffer before reading more (this may also
    //       involve increasing the chunk size substantially to be more efficient)
    // If we find a null byte outside of the header
    if let Some(i) = bytes[8..].iter().position(|b| b == &0x00) {
      // If there's not enough data to check if we have a full message, add all the data back to the
      // buffer and start again.
      if i + 8 + 8 >= bytes.len() {
        buffer.append(&mut bytes);
        continue;
      }
      // At this point, we know we're somewhere in the header, but not where. To account for the
      // possibility of both null bytes being in the timestamp and colons being in the header, we
      // assume here that the last byte of the header is always 0x00, which is an unsafe assumption,
      // but always seems to be true.
      // Find the rightmost null byte
      let last_null = match bytes[i + 8 .. i + 8 + 8].iter().rposition(|b| b == &0x00) {
        Some(n) => n,
        None => 0 // we're already at the null byte
      };
      // The colon's index will be next to the null byte
      let colon = i + 8 + last_null + 1;
      // If we found a colon at the assumed index
      if bytes[colon] == 0x3a {
        // Add the message to the message vector
        messages.push(bytes[..colon - 8].to_vec());
        // Add the rest of the bytes back to the buffer
        buffer.append(&mut bytes[colon - 8..].to_vec());
      // If we didn't find a colon, which is indicative of being at the end of the log
      } else {
        // FIXME: this will keep reading into empty memory
        messages.push(bytes[..i + 8].to_vec());
        buffer.append(&mut bytes[i + 8..].to_vec());
        break; // TODO: periodically check the end of the memory chunk for new messages
      }
    // If we don't find a null byte outside of the header
    } else {
      // Add all the bytes back to the buffer and start again
      buffer.append(&mut bytes);
    }
  }
  for message in messages {
    let raw_entry = RawEntry::new(message);
    let parts = raw_entry.as_parts().unwrap();
    let entry = parts.as_entry();
    println!("[{}], {}, <{}> {}", entry.timestamp, entry.entry_type, entry.sender.map(|x| x.display_text()).unwrap_or_else(|| String::from("None")), entry.message.display_text());
    println!("raw: {}", to_hex_string(&raw_entry.bytes));
    println!("message: {}", to_hex_string(&parts.message));
    println!("message: {}", String::from_utf8_lossy(&parts.message));
    println!("message: {:#?}", entry.message);
    println!();
}
}
