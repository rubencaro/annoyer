//! This is the module for the Client logic.
//!
//! This allows to set up a vector of future loops that can run indefinitely.
//! See `build_client_loop` docs for details.

use super::config;
use futures::future::{join_all, loop_fn, ok, Future, Loop};
use hyper::client::connect::HttpConnector;
use hyper::Body;
use hyper::Client as HC;
use std::io::Error;
use std::sync::mpsc::Sender;

/// The individual Client internal state
#[derive(Debug)]
pub struct Client {
    pub ping_count: u8,
    pub error_count: u8,
    pub hyper_client: HC<HttpConnector, Body>,
}

impl Client {
    /// Create an empty Client struct
    pub fn new() -> Self {
        Client {
            ping_count: 0,
            error_count: 0,
            hyper_client: HC::new(),
        }
    }

    /// This client is done
    pub fn is_done(self) -> bool {
        self.ping_count + self.error_count >= 5
    }
}

/// Build the futures struct to be run.
///
/// This returns a `JoinAll` of a vector of `concurrency` members of `LoopFn`
/// each executing a separate `Client` sequence of calls.
///
/// That means that when you run this with `tokio::run()` you'll have
/// `concurrency` independent loops running concurrently on `tokio`'s thread pool.
/// Each of them will be an independent requesting `Client` generating load over
/// configured targets.
///
pub fn build_client_loop<'a>(
    conf: config::Config,
    tx: Sender<&'a str>,
) -> Box<Future<Item = (), Error = ()> + Send + 'a> {
    let mut parallel = Vec::new();
    for _i in 0..conf.concurrency {
        let tx2 = tx.clone();
        let url = conf.url.clone();
        let ping_til_done = loop_fn(Client::new(), move |mut c| {
            let tx3 = tx2.clone();
            c.hyper_client.get(url.clone())
            .map(|res| {
                tx3.send("hey").unwrap();
                println!("Response: {}", res.status());
                c.ping_count += 1;
            })
            .map_err(|err| {
                println!("Error: {}", err);
                c.error_count += 1;
            });

            if c.is_done() {
                Ok(Loop::Break(c))
            } else {
                Ok(Loop::Continue(c))
            }
        });
        parallel.push(ping_til_done);
    }

    let all = join_all(parallel).then(|res: Result<Vec<Client>, Error>| {
        println!("{:?}", res);
        ok(())
    });

    Box::new(all)
}
