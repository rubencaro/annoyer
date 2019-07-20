extern crate futures;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate tokio;

use futures::future::{join_all, loop_fn, ok, Future, FutureResult, Loop};
use std::io::Error;
use std::sync::mpsc::{self, Receiver, Sender};
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
        drop(tx);
        ok((self, done))
    }
}

fn main() {
    let matches = build_matches();
    let concurrency = parse_u32(&matches, "concurrency", 10);
    let url = parse_url(&matches, "url");
    println!("url: {}", url);

    let (tx, rx) = mpsc::channel();
    let work = define_work(concurrency, tx);
    let handler = thread::spawn(|| collector(rx));

    tokio::run(work);
    handler.join().unwrap();
}

fn define_work<'a>(
    concurrency: u32,
    tx: Sender<&'a str>,
) -> Box<Future<Item = (), Error = ()> + Send + 'a> {
    let mut parallel = Vec::new();
    for _i in 0..concurrency {
        let tx2 = tx.clone();
        let ping_til_done = loop_fn(Client::new(), move |c| {
            let tx3 = tx2.clone();
            c.send_ping()
                .and_then(|client| client.receive_pong(tx3))
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
    drop(tx);

    let all = join_all(parallel).then(|res| {
        println!("{:?}", res);
        ok(())
    });

    Box::new(all)
}

fn collector(rx: Receiver<&str>) {
    let mut total = Client::new();
    for r in rx {
        total.ping_count += 1;
        println!("Got: {}", r);
        println!("Total: {:?}", total);
    }
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

fn build_matches() -> clap::ArgMatches<'static> {
    clap::App::new("Annoyer")
        .version("0.1.0")
        .about("Annoying HTML load generator.")
        .args_from_usage(
            "-u, --url <string> 'The URL to be called'
             -c, --concurrency <number> 'Indicates the number of parallel workers'",
        )
        .get_matches()
}
