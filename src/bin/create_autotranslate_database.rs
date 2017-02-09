#![feature(range_contains)]

extern crate csv;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use csv::Reader;

use std::env::args;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::ops::Range;

fn main() {
  let args: Vec<String> = args().skip(1).collect();

  if args.is_empty() {
    println!("Specify the directory where the EXH CSVs are located.");
    return;
  }

  let directory_path = Path::new(&args[0]);
  if !directory_path.exists() {
    println!("The directory does not exist.");
    return;
  }
  if !directory_path.is_dir() {
    println!("The directory is not a directory.");
    return;
  }

  let mut db = Database {
    directory_path: directory_path.to_owned(),
    completions: Vec::new()
  };
  db.read_completions();
  println!("{}", serde_json::to_string(&db.completions).unwrap());
}

#[derive(Debug)]
struct Database {
  directory_path: PathBuf,
  completions: Vec<Completion>
}

impl Database {
  fn read_completions(&mut self) {
    let mut reader = Reader::from_file(self.directory_path.join("completion.exh_en.csv")).unwrap().has_headers(true);
    type Row = (u64, u64, String, String, String);
    let rows = reader.decode().collect::<csv::Result<Vec<Row>>>().unwrap();
    for row in rows {
      let location = row.2;
      if !location.is_empty() {
        if location == "@" {
          continue;
        }
        let location = Location::from_descriptor(location);
        let mut extra_completions = self.read_completions_from(row.1, location);
        self.completions.append(&mut extra_completions);
        continue;
      }
      self.completions.push(Completion {
        category: row.1,
        id: row.0,
        value: row.3
      });
    }
  }

  fn read_completions_from(&self, category: u64, location: Location) -> Vec<Completion> {
    let loc_path = self.directory_path.join(&location.file_name);
    if !loc_path.exists() {
      writeln!(std::io::stderr(), "{} does not exist", location.file_name).unwrap();
      return Vec::new();
    }
    let mut reader = Reader::from_file(loc_path).unwrap().has_headers(true);
    let rows = reader.records().collect::<csv::Result<Vec<Vec<String>>>>().unwrap();
    let mut completions = Vec::new();
    for row in rows {
      let id = row[0].parse::<u64>().unwrap();
      if let Some(ref w) = location.indices {
        if !w.contains(id) {
          continue;
        }
      }
      let column = if let Some(ref w) = location.indices {
        w.column as usize + 1
      } else {
        1
      };
      let value = row[column].clone();
      if value.is_empty() {
        continue;
      }
      completions.push(Completion {
        category: category,
        id: id,
        value: value
      });
    }
    completions
  }
}

#[derive(Debug, Serialize)]
struct Completion {
  category: u64,
  id: u64,
  value: String
}

#[derive(Debug)]
struct Which {
  column: u64,
  ranges: Vec<Range<usize>>
}

impl Which {
  fn from_descriptor(descriptor: String) -> Which {
    let split: Vec<&str> = descriptor.split(',').collect();
    let mut column = 0;
    let mut ranges = Vec::new();
    for s in split {
      if !s.contains('-') {
        continue;
      }
      let parts: Vec<&str> = s.split('-').collect();
      if parts[0] == "col" {
        column = parts[1].parse::<u64>().unwrap();
        continue;
      }
      let start: usize = parts[0].parse().unwrap();
      let end: usize = parts[1].parse().unwrap();
      let range = Range { start: start, end: end + 1 };
      ranges.push(range);
    }
    Which {
      column: column,
      ranges: ranges
    }
  }

  fn contains(&self, n: u64) -> bool {
    self.ranges.iter().any(|x| x.contains(n as usize))
  }
}

#[derive(Debug)]
struct Location {
  file_name: String,
  indices: Option<Which>
}

impl Location {
  fn from_descriptor(descriptor: String) -> Location {
    let split: Vec<&str> = descriptor.split('[').collect();
    let file_name = format!("{}.exh_en.csv", split[0].to_lowercase());
    let indices = if split.len() == 1 {
      None
    } else {
      let d = split[1].replace("]", "");
      Some(Which::from_descriptor(d))
    };
    Location {
      file_name: file_name,
      indices: indices
    }
  }
}
