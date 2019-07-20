//! This is the module for the Collector logic.
//!
//! This is merely a loop reading from a channel until it's close (if it ever is),
//! and storing stats on its internal state.

use std::sync::mpsc::Receiver;

/// The internal Collector state
#[derive(Debug)]
struct Collector {
  ping_count: u8,
}

impl Collector {
  /// Create an empty Collector
  fn new() -> Self {
    Collector { ping_count: 0 }
  }
}

/// The main collect loop.
/// This is meant to be run on a separate thread, as it will block until given channel is closed.
pub fn collect(rx: Receiver<&str>) {
  let mut total = Collector::new();
  for r in rx {
    total.ping_count += 1;
    println!("Got: {}", r);
    println!("Total: {:?}", total);
  }
}
