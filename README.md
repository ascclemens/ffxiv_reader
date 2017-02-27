# ffxiv_reader

This is a FFXIV in-memory log reader written in [Rust](http://rust-lang.org/).

## Warning

As it stands, `ffxiv_reader` is only built for the macOS version of FFXIV. I have recently updated
`memreader` to support both Linux and Windows in addition to macOS, but I have not updated
`ffxiv_reader` with proper addresses for those OSes.

I'm pretty sure the way I'm handling getting the base address used is incorrect for macOS anyway.

## Usage

`MemoryEntryReader` is the main entry point for reading from the memory.

```rust
extern crate ffxiv_reader;

use ffxiv_reader::MemoryEntryReader;

fn main() {
  // Read all the entries currently in memory and exit.
  let reader = MemoryEntryReader::new(some_pid, true);
  for entry in reader.iter() {
    println!("{:#?}", entry);
  }
}
```

## Entries

Each entry in the log is made up of several components: a timestamp, a sender, and a message.

The sender and the message are both made up of parts. The sender is one part, and the message can
be made up of multiple parts, each combining to form the display text of the message.

## Autotranslate

The FFXIV autotranslate system is contained in the game's internal files, which can be read using
[FFXIV Explorer](http://ffxivexplorer.fragmenterworks.com/). The instructions for dumping the data
and creating the JSON database used by `ffxiv_reader` are contained in
`src/bin/create_autotranslate_database.rs`.

`AutoTranslatePart::get_completion` can be used to query the database. The actual file does not need
to be present on the filesystem, as it is included (gzipped) when the library is compiled. It is
only loaded into memory when it is queried.
