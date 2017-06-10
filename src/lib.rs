extern crate byteorder;
extern crate memreader;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use byteorder::{LittleEndian, ByteOrder};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::Read;
use memreader::prelude::*;

macro_rules! opt {
  ($e:expr) => (opt_or_else!($e, None))
}

macro_rules! opt_or {
  ($e:expr, $ret:expr) => {{
    match $e {
      Some(x) => x,
      None => $ret
    }
  }}
}

macro_rules! opt_or_else {
  ($e:expr, $ret:expr) => (opt_or!($e, return $ret))
}

macro_rules! try_or {
  ($e:expr, $ret:expr) => {{
    #[allow(unused_variables)]
    match $e {
      Ok(x) => x,
      Err(e) => $ret
    }
  }}
}

macro_rules! read {
  ($e:expr, $n:expr) => {{
    let mut buf = vec![0; $n];
    $e.read_exact(&mut buf).map(|_| buf)
  }}
}

pub mod messages;

use messages::entries::{Entry, RawEntry};

fn get_base_address(reader: Option<&MemReader>) -> Option<usize> {
  let reader = opt!(reader);
  reader.base_address("ffxiv.exe").ok()
}

fn get_lines_address(reader: Option<&MemReader>) -> Option<usize> {
  let reader = opt!(reader);
  let base_address = opt!(get_base_address(Some(reader)));
  let pointer_1 = base_address + 0x0107E3F0;
  let value_1 = try_or!(read!(reader.address_slice_len(pointer_1, 4), 4), return None);
  let pointer_2 = LittleEndian::read_u32(&value_1) as usize + 0x18;
  let value_2 = try_or!(read!(reader.address_slice_len(pointer_2, 4), 4), return None);
  Some(LittleEndian::read_u32(&value_2) as usize + 0x2b8)
}

fn get_chat_pointer(lines_address: usize) -> usize {
  lines_address + 52
}

fn get_index_pointer(lines_address: usize) -> usize {
  lines_address + 40
}

// TODO: Handle the game closing, logging out, disconnects, etc. better. Wait for pointers to become
//       valid again, then start reading again.

fn to_hex_string(bytes: &[u8]) -> String {
  bytes.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" ")
}

fn read_var_le(bytes: &[u8]) -> Option<u64> {
  if bytes.len() == 1 {
    return Some(bytes[0] as u64);
  } else if bytes.is_empty() || bytes.len() > 8 || bytes.len() % 2 == 1 {
    return None;
  }
  let mut res: u64 = 0;
  for (i, byte) in bytes.iter().enumerate() {
    res |= (*byte as u64) << (8 * i);
  }
  Some(res)
}

/// A reader that extracts [`Entries`](messages/entries/struct.Entry.html) from FFXIV's memory.
///
/// Ideally, this is used as an iterator. As an iterator, it iterates over all messages in the
/// memory until no more are found when `next()` is called. If `stop` (in `new` below) is `true`,
/// then `next()` will return `None` at this point. Otherwise, it will block until a new message is
/// found in the memory.
///
/// When blocking, the `stop()` method should be used when breaking out of the loop and not dropping
/// the reader, or else the reader will continue to store messages in memory.
///
/// # Examples
/// This pattern will block and iterate forever.
///
/// ```rust,no_run
/// let reader = MemoryEntryReader::new(my_pid, false);
/// for entry in reader.iter() {
///   println!("{:?}", entry);
/// }
/// ```
pub struct MemoryEntryReader {
  pub pid: u32,
  pub stop: bool,
  run: Arc<AtomicBool>
}

impl MemoryEntryReader {
  /// Create a new reader.
  ///
  /// Reads from the process using PID `pid`. `stop` denotes whether the reader will stop once it
  /// runs out of messages.
  pub fn new(pid: u32, stop: bool) -> Self {
    MemoryEntryReader {
      pid: pid,
      stop: stop,
      run: Arc::new(AtomicBool::new(false))
    }
  }

  /// Starts the memory reading loop.
  ///
  /// This is automatically called when the reader is used as an iterator. This can be used when not
  /// using a loop to get the raw bytes for each entry from the `Receiver`.
  ///
  /// This will return `None` if `start` has already been called or if the memory can't be read.
  pub fn start(&self) -> Option<Receiver<Vec<u8>>> {
    if self.run.load(Ordering::Relaxed) {
      return None;
    }
    // Create a reader around the PID of the game.
    let reader = match MemReader::new(self.pid) {
      Ok(r) => r,
      Err(e) => {
        println!("Encountered error {} when trying to access memory.", e);
        return None;
      }
    };
    let lines_address = opt!(get_lines_address(Some(&reader)));
    let chat_pointer = get_chat_pointer(lines_address);
    let index_pointer = get_index_pointer(lines_address);
    let raw_chat_pointer = opt!(read!(reader.address_slice_len(chat_pointer, 4), 4).ok());
    let chat_address = LittleEndian::read_u32(&raw_chat_pointer) as usize;
    let stop = self.stop;
    let (tx, rx) = std::sync::mpsc::channel();
    self.run.store(true, Ordering::Relaxed);
    let run = self.run.clone();
    std::thread::spawn(move || {
      // Index of last read index
      let mut index_index = 0;
      'main_loop: while run.load(Ordering::Relaxed) {
        // Get raw bytes for current index pointer
        let raw_pointer = try_or!(read!(reader.address_slice_len(index_pointer, 4), 4), break);
        // Read the raw bytes into an address
        let pointer = LittleEndian::read_u32(&raw_pointer);
        // Read the total number of lines (modulo 1000 because the game wraps around at 1000)
        let num_lines = {
          let raw = try_or!(read!(reader.address_slice_len(lines_address, 4), 4), break);
          LittleEndian::read_u32(&raw) % 1000
        };
        // Read u32s backwards until we hit 0
        let mut mem_indices = Vec::with_capacity(index_index + 1);
        loop {
          // If the amount of lines we've read is equal to the number of lines, break
          if mem_indices.len() == num_lines as usize {
            break;
          }
          // Read backwards, incrementing by four for each index read
          let raw_index = try_or!(read!(reader.address_slice_len(pointer as usize - (4 * (mem_indices.len() + 1)), 4), 4), break);
          // Read the raw bytes into a u32
          let index = LittleEndian::read_u32(&raw_index);
          // Otherwise, insert the index at the start
          mem_indices.insert(0, index);
        }
        // If the number of indices we just read is equal to the last index of the indices we read,
        // there are no new messages, so sleep and restart the loop.
        if mem_indices.len() == index_index {
          if stop {
            break;
          } else {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
          }
        } else if mem_indices.len() < index_index {
          // If the amount of indices we've read is less than the amount we were at last time,
          // we've wrapped around in the memory, so reset the index to 0.
          index_index = 0;
        }
        // Get all the new indices
        let new_indices = &mem_indices[index_index..];
        // Get the last index, or 0 to start
        let mut last_index = if index_index == 0 {
          0
        } else {
          // The last index will be in the new indices we just read, being the last one we have read
          mem_indices[index_index - 1]
        };
        index_index = mem_indices.len();
        // Read each new message and send it
        for index in new_indices {
          let read_size = *index as usize - last_index as usize;
          let message = try_or!(read!(reader.address_slice_len(chat_address + last_index as usize, read_size), read_size), break);
          last_index = *index;
          try_or!(tx.send(message), break 'main_loop);
        }
      }
    });
    Some(rx)
  }

  /// Stops the memory loop.
  ///
  /// This is called automatically when the reader is dropped.
  pub fn stop(&self) {
    self.run.store(false, Ordering::Relaxed);
  }

  /// Creates an iterator from this reader.
  ///
  /// This automatically calls `start`.
  ///
  /// If `start` has been called but `stop` has not been called, the iterator returned will always
  /// return `None`.
  pub fn iter(&self) -> MemoryEntryReaderIterator {
    MemoryEntryReaderIterator { rx: self.start() }
  }
}

impl Drop for MemoryEntryReader {
  fn drop(&mut self) {
    self.stop();
  }
}

/// The iterator for [`MemoryEntryReader`](struct.MemoryEntryReader.html).
///
/// See [`MemoryEntryReader`](struct.MemoryEntryReader.html) for more information.
pub struct MemoryEntryReaderIterator {
  rx: Option<Receiver<Vec<u8>>>
}

impl Iterator for MemoryEntryReaderIterator {
  type Item = Entry;

  fn next(&mut self) -> Option<Entry> {
    let rx = match self.rx {
      Some(ref r) => r,
      None => return None
    };
    let bytes = match rx.recv() {
      Ok(b) => b,
      Err(_) => return None
    };
    let raw = RawEntry::new(bytes);
    let parts = opt!(raw.as_parts());
    let entry = parts.as_entry();
    Some(entry)
  }
}
