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

use clap::{App, Arg};
use futures::future;
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::net::SocketAddr;

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

fn run<'a, 'b>(bot_token: &'a str, listening_address: &'b str) {
    let addr: SocketAddr = listening_address.parse().unwrap();

    let server = Server::bind(&addr)
        .serve(|| service_fn(echo))
        .map_err(|e| error!("server error: {}", e));

    info!("Listening on http://{}", addr);
    rt::run(server);
}

fn echo(req: Request<Body>) -> impl Future<Item = Response<Body>, Error = hyper::Error> + Send {
    let mut response = Response::new(Body::empty());
    future::ok(response)
}
