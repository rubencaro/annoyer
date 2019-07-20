#[macro_use]
extern crate clap;

use std::sync::mpsc;
use std::thread;

mod client;
mod collector;
mod config;

fn main() {
    let conf = config::get_config();
    println!("url: {:?}", conf);

    let (tx, rx) = mpsc::channel();
    let work = client::build_client_loop(conf.concurrency, tx);
    let handler = thread::spawn(|| collector::collect(rx));

    tokio::run(work);
    handler.join().unwrap();
}
