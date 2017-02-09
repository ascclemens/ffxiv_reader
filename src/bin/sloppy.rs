extern crate ffxiv_reader;
extern crate serde_json;
extern crate time;

use ffxiv_reader::MemoryEntryReader;
use ffxiv_reader::messages::parts::Part;
use ffxiv_reader::messages::MessageType;
use time::Timespec;

use std::env::args;
use std::process::Command;

fn main() {
  // Gather the arguments supplied to the program.
  let args: Vec<String> = args().skip(1).collect();
  // Ensure they are not empty.
  if args.len() < 2 {
    println!("Please supply a PID and device ID.");
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
  // Attempt to parse a device ID from the second arg.
  let device_id: u32 = match args[1].parse() {
    Ok(i) => i,
    Err(e) => {
      println!("Invalid Device ID: {}.", e);
      return;
    }
  };
  // Check whether the program should continue scanning memory or just stop.
  let stop = if args.len() > 2 {
    match args[2].to_lowercase().parse() {
      Ok(b) => b,
      Err(e) => {
        println!("Invalid stop argument. Please specify true/false. {}", e);
        return;
      }
    }
  } else { false };
  // Create a log reader.
  let reader = MemoryEntryReader::new(pid, stop);
  // Record program start time, so it doesn't replay deaths.
  let start_time = time::now();
  // Last sloppy time
  let mut last_sloppy: Option<time::Tm> = None;
  // Loop over every old and new entry
  for entry in reader {
    // Skip anything that's not a death
    if entry.message_type != MessageType::BattleSystemMessages && entry.message_type != MessageType::BattleDeath {
      continue;
    }
    // Find the first plain text part.
    let plain_text_part = entry.message.parts.iter()
      .find(|x| if let Part::PlainText(_) = **x { true } else { false });
    // Get the text from the part.
    let text = match plain_text_part {
      Some(&Part::PlainText(ref m)) => m,
      _ => continue
    };
    // Check for the player being defeated or someone else being defeated.
    if !text.starts_with("You are defeated by ") && !text.starts_with(" is defeated by ") {
      continue;
    }
    // Get the time of the message.
    let t = time::at(Timespec::new(entry.timestamp as i64, 0));
    // If it was before the application started, skip it.
    if t - start_time <= time::Duration::zero() {
      continue;
    }
    // If a sloppy has been played before
    if let Some(last) = last_sloppy {
      // Check to make sure 30 seconds have passed before playing another.
      if t - last <= time::Duration::seconds(30) {
        continue;
      }
    }
    // Update last sloppy time.
    last_sloppy = Some(time::now());
    // Play it once over the microphone device and don't wait for the process to end.
    let _ = Command::new("mplayer")
      .arg("-ao")
      .arg(&format!("coreaudio:device_id={}", device_id))
      .arg("/Users/kyleclemens/Downloads/quiet_sloppy.mp3")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .spawn();
    // Play it once over the speakers and wait for the process to end.
    let _ = Command::new("mplayer")
      .arg("/Users/kyleclemens/Downloads/quiet_sloppy.mp3")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .status();
  }
}
