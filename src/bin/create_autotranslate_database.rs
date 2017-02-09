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

const LANGS: &'static [Lang] = &[Lang::German, Lang::English, Lang::French, Lang::Japanese];

// Use a dump tool like FFXIV Data Explorer and open 0a0000.win32.index. Dump the EXDs inside to
// CSVs. Rename the files so each file contains only the data after the last backslash.
// "a\exd\emotemode.exh.csv" becomes "emotemode.exh.csv"

// Provide the folder where all of the CSVs are stored to this binary. It will output JSON
// for every auto-translate entry.

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

  let dbs: Vec<SingleLangDatabase> = LANGS.iter().map(|l| {
      let mut db = SingleLangDatabase {
        directory_path: directory_path.to_owned(),
        completions: Vec::new(),
        language: l.clone()
      };
      db.read_completions();
      db
  })
  .collect();
  let db = Database::from_single_lang(dbs).unwrap();
  println!("{}", serde_json::to_string(&db.completions).unwrap());
}

#[derive(Debug)]
struct Database {
  completions: Vec<Completion>
}

impl Database {
  fn from_single_lang(single: Vec<SingleLangDatabase>) -> Option<Database> {
    if single.is_empty() {
      return None;
    }
    let first = &single[0];
    let mut new_completions: Vec<Completion> = first.completions.iter().map(|x| Completion {
      category: x.category,
      id: x.id,
      values: CompletionValues {
        en: if first.language == Lang::English { x.value.clone() } else { String::new() },
        de: if first.language == Lang::German { x.value.clone() } else { String::new() },
        fr: if first.language == Lang::French { x.value.clone() } else { String::new() },
        ja: if first.language == Lang::Japanese { x.value.clone() } else { String::new() },
      }
    })
    .collect();
    for db in &single[1..] {
      let db_language = db.language.clone();
      // let db_completions = db.completions;
      for c in &db.completions {
        let mut completion = new_completions.iter_mut().find(|x| x.id == c.id && x.category == c.category).unwrap();
        match db_language {
          Lang::English => completion.values.en = c.value.clone(),
          Lang::German => completion.values.de = c.value.clone(),
          Lang::French => completion.values.fr = c.value.clone(),
          Lang::Japanese => completion.values.ja = c.value.clone(),
        }
      }
    }
    Some(Database {
      completions: new_completions
    })
  }
}

#[derive(Debug, PartialEq, Clone)]
enum Lang {
  English,
  German,
  French,
  Japanese
}

impl Lang {
  fn code(&self) -> &str {
    match *self {
      Lang::English => "en",
      Lang::German => "de",
      Lang::French => "fr",
      Lang::Japanese => "ja"
    }
  }
}

#[derive(Debug)]
struct SingleLangDatabase {
  directory_path: PathBuf,
  completions: Vec<SingleLangCompletion>,
  language: Lang
}

impl SingleLangDatabase {
  fn read_completions(&mut self) {
    let file_name = format!("completion.exh_{}.csv", self.language.code());
    let mut reader = Reader::from_file(self.directory_path.join(&file_name)).unwrap().has_headers(true);
    type Row = (u64, u64, String, String, String);
    let rows = reader.decode().collect::<csv::Result<Vec<Row>>>().unwrap();
    for row in rows {
      let location = row.2;
      if !location.is_empty() {
        if location == "@" {
          continue;
        }
        let location = Location::from_descriptor(location, &self.language);
        let mut extra_completions = self.read_completions_from(row.1, location);
        self.completions.append(&mut extra_completions);
        continue;
      }
      self.completions.push(SingleLangCompletion {
        category: row.1,
        id: row.0,
        value: row.3
      });
    }
  }

  fn read_completions_from(&self, category: u64, location: Location) -> Vec<SingleLangCompletion> {
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
      completions.push(SingleLangCompletion {
        category: category,
        id: id,
        value: value
      });
    }
    completions
  }
}

#[derive(Debug, Serialize)]
struct SingleLangCompletion {
  category: u64,
  id: u64,
  value: String
}

#[derive(Debug, Serialize)]
struct Completion {
  category: u64,
  id: u64,
  values: CompletionValues
}

#[derive(Debug, Serialize)]
struct CompletionValues {
  en: String,
  de: String,
  fr: String,
  ja: String
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
  fn from_descriptor(descriptor: String, lang: &Lang) -> Location {
    let split: Vec<&str> = descriptor.split('[').collect();
    let file_name = format!("{}.exh_{}.csv", split[0].to_lowercase(), lang.code());
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
