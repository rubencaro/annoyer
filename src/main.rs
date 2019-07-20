#[macro_use]
extern crate clap;

use std::sync::mpsc;
use std::thread;

mod client;
mod collector;
mod config;

fn main() {
    // First of all get config data
    let conf = config::get_config();
    println!("url: {:?}", conf);

    // Create the communication channel
    let (tx, rx) = mpsc::channel();
    // Build the client loop futures passing them the sending side of the channel
    let work = client::build_client_loop(conf.concurrency, tx);
    // Pass the receiving side to the stats collector and spawn it on a separate thread
    let handler = thread::spawn(|| collector::collect(rx));
    // Run the client loop futures
    tokio::run(work);
    // Wait for the collector thread to end
    handler.join().unwrap();
}
