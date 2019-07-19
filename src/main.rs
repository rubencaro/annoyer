extern crate futures;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate tokio;

use futures::future::{join_all, loop_fn, ok, Future, FutureResult, Loop};
use std::io::Error;
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Debug)]
struct Client {
    ping_count: u8,
}

impl Client {
    fn new() -> Self {
        Client { ping_count: 0 }
    }

    fn send_ping(self) -> FutureResult<Self, Error> {
        println!("{:?}", self);
        ok(Client {
            ping_count: self.ping_count + 1,
        })
    }

    fn receive_pong(self, tx: Sender<&str>) -> FutureResult<(Self, bool), Error> {
        let done = self.ping_count >= 5;
        tx.send("hey").unwrap();
        ok((self, done))
    }
}

fn main() {
    let matches = clap::App::new("Annoyer")
        .version("0.1.0")
        .about("Annoying HTML load generator.")
        .args_from_usage(
            "-u, --url <string> 'The URL to be called'
             -c, --concurrency <number> 'Indicates the number of parallel workers'",
        )
        .get_matches();

    let concurrency = parse_u32(&matches, "concurrency", 10);
    let url = parse_url(&matches, "url");
    println!("url: {}", url);

    let (tx, rx) = mpsc::channel();

    let mut parallel = Vec::new();
    for _i in 0..concurrency {
        let tx = tx.clone();
        let ping_til_done = loop_fn(Client::new(), move |client| {
            let tx = tx.clone();
            client
                .send_ping()
                .and_then(|client| client.receive_pong(tx))
                .and_then(|(client, done)| {
                    if done {
                        Ok(Loop::Break(client))
                    } else {
                        Ok(Loop::Continue(client))
                    }
                })
        });
        parallel.push(ping_til_done);
    }
    let work = join_all(parallel).then(|res| {
        println!("{:?}", res);
        ok(())
    });

    thread::spawn(move || {
        for r in rx {
            println!("Got: {}", r);
        }
    });

    tokio::run(work);
}

fn parse_u32(matches: &clap::ArgMatches<'_>, name: &str, default: u32) -> u32 {
    value_t!(matches, name, u32).unwrap_or_else(|e| {
        println!("{}\nUsing default one.", e);
        default
    })
}

fn parse_url(matches: &clap::ArgMatches<'_>, name: &str) -> hyper::Uri {
    matches
        .value_of(name)
        .unwrap()
        .parse::<hyper::Uri>()
        .unwrap()
}
