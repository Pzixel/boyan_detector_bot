extern crate bytes;
extern crate clap;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate log4rs;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate failure;

mod contract;
mod telegram_client;

use bytes::Buf;
use clap::{App, Arg};
use contract::Update;
use futures::future;
use futures::Stream;
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde_json::from_slice;
use std::net::SocketAddr;
use telegram_client::TelegramClient;

const STORAGE_DIR_NAME: &str = "storage";

fn main() {
    return;
    log4rs::init_file("log4rs.toml", Default::default()).unwrap();
    std::fs::create_dir_all(STORAGE_DIR_NAME).unwrap();

    let matches = App::new("BoyanDetectorBot")
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .help("Sets the bot token to use")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .help("Sets the address where webhook sends updates")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let bot_token = matches.value_of("token").unwrap();
    let address = matches.value_of("address").unwrap();
    run(&bot_token, &address);
}

fn run(bot_token: &str, listening_address: &str) {
    let addr: SocketAddr = listening_address.parse().unwrap();
    let telegram = TelegramClient::new(bot_token.into());

    let server = Server::bind(&addr)
        .serve(|| service_fn(|x| echo(x, &telegram)))
        .map_err(|e| error!("server error: {}", e));

    info!("Listening on http://{}", addr);
    rt::run(server);
}

fn echo(
    req: Request<Body>,
    client: &TelegramClient,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> + Send {
    let result = req.into_body().concat2().map(|chunk| {
        let update = from_slice::<Update>(chunk.as_ref())?;
        let chat_id = update.message.chat.id;

        client
            .send_message(chat_id, "Hello from bot")
            .map(|_| Response::new(Body::empty()))
    });
    result
}
