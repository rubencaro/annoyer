use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct Collector {
  pub ping_count: u8,
}

impl Collector {
  pub fn new() -> Self {
    Collector { ping_count: 0 }
  }
}

pub fn collect(rx: Receiver<&str>) {
  let mut total = Collector::new();
  for r in rx {
    total.ping_count += 1;
    println!("Got: {}", r);
    println!("Total: {:?}", total);
  }
}
