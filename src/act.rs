use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread::sleep;
use std::time::Duration;
use std::thread;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::path::{Path, PathBuf};

use chrono::DateTime;

use messages::{Message, MessageType, Parses};
use messages::parser::MessageParser;
use messages::parts::NamePart;
use messages::entries::Entry;

pub struct ActReader {
  path: PathBuf,
  stop: bool,
  run: Arc<AtomicBool>
}

impl ActReader {
  pub fn new<P: AsRef<Path>>(path: P, stop: bool) -> ActReader {
    ActReader {
      path: path.as_ref().to_path_buf(),
      stop,
      run: Arc::new(AtomicBool::new(false))
    }
  }

  pub fn start(&self) -> Option<Receiver<Entry>> {
    if self.run.load(Ordering::Relaxed) {
      return None;
    }
    let f = match File::open(&self.path).ok() {
      Some(f) => f,
      None => return None
    };
    let (tx, rx) = channel();
    let mut reader = BufReader::new(f);
    let stop = self.stop;
    thread::spawn(move || {
      let mut content = String::new();
      while let Ok(size) = reader.read_line(&mut content) {
        if size == 0 {
          if stop {
            break;
          } else {
            sleep(Duration::from_millis(100));
            continue;
          }
        }

        let mut parts = content.split('|').skip(1);
        let timestamp_str = parts.next().and_then(|x| DateTime::parse_from_rfc3339(x).ok());
        // yyyy-MM-dd'T'HH:mm:ss.SSSSSSSXXX
        // 2017-09-29T11:55:01.5120000-04:00
        // %Y-%m-%dT%H:%M:%S.%.7f%:z
        let datetime = match timestamp_str {
          Some(dt) => dt,
          None => continue
        };
        let timestamp = datetime.timestamp() as u32;

        let kind = parts.next().and_then(|x| u8::from_str_radix(x, 16).ok());
        let message_type = match kind {
          Some(k) => MessageType::from(k),
          None => continue
        };

        let sender = parts.next().and_then(|x| NamePart::parse(x.as_bytes()));

        let left_over: Vec<_> = parts.collect();
        let message_str = left_over[..left_over.len() - 1].join("|");
        let message_parts = MessageParser::parse(message_str.as_bytes());
        let message = Message::new(message_parts);

        tx.send(Entry {
          message_type,
          timestamp,
          sender,
          message
        }).unwrap();
      }
    });
    Some(rx)
  }
}
