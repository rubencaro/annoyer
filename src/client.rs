//! This is the module for the Client logic.
//!
//! This allows to set up a vector of future loops that can run indefinitely.
//! See `build_client_loop` docs for details.

use futures::future::{join_all, loop_fn, ok, Future, FutureResult, Loop};
use std::io::Error;
use std::sync::mpsc::Sender;

/// The individual Client internal state
#[derive(Debug)]
pub struct Client {
    pub ping_count: u8,
}

impl Client {
    /// Create an empty Client struct
    pub fn new() -> Self {
        Client { ping_count: 0 }
    }

    /// Send a ping request
    pub fn send_ping(self) -> FutureResult<Self, Error> {
        println!("{:?}", self);
        ok(Client {
            ping_count: self.ping_count + 1,
        })
    }

    /// Receive the pong response
    pub fn receive_pong(self, tx: Sender<&str>) -> FutureResult<(Self, bool), Error> {
        let done = self.ping_count >= 5;
        tx.send("hey").unwrap();
        ok((self, done))
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

    let all = join_all(parallel).then(|res| {
        println!("{:?}", res);
        ok(())
    });

    Box::new(all)
}
