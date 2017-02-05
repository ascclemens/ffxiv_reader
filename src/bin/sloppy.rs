extern crate ffxiv_reader;
extern crate serde_json;
extern crate time;

use ffxiv_reader::*;
use time::Timespec;

use std::env::args;
use std::process::Command;

fn main() {
  // Gather the arguments supplied to the program.
  let args: Vec<String> = args().skip(1).collect();
  // Ensure they are not empty.
  if args.len() < 2 {
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
  let reader = FfxivMemoryLogReader::new(pid, stop);
  // Record program start time, so it doesn't replay deaths.
  let start_time = time::now();
  // Last sloppy time
  let mut last_sloppy: Option<time::Tm> = None;
  // Print out every entry.
  for entry in reader {
    if entry.message_type != MessageType::BattleDeathRevive && entry.message_type != MessageType::BattleDeath {
      continue;
    }
    let plain_text_part = entry.message.parts.iter()
      .find(|x| if let Part::PlainText(_) = **x { true } else { false });
    let text = match plain_text_part {
      Some(&Part::PlainText(ref m)) => m,
      _ => continue
    };
    if !text.starts_with("You are defeated by ") && !text.starts_with(" is defeated by ") {
      continue;
    }
    let t = time::at(Timespec::new(entry.timestamp as i64, 0));
    if t - start_time <= time::Duration::zero() {
      continue;
    }
    if let Some(last) = last_sloppy {
      if t - last <= time::Duration::seconds(30) {
        continue;
      }
    }
    last_sloppy = Some(time::now());
    let _ = Command::new("mplayer")
      .arg("-ao")
      .arg(&format!("coreaudio:device_id={}", device_id))
      .arg("/Users/kyleclemens/Downloads/quiet_sloppy.mp3")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .spawn();
    let _ = Command::new("mplayer")
      .arg("/Users/kyleclemens/Downloads/quiet_sloppy.mp3")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .status();
  }
}
