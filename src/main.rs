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
extern crate url;

#[macro_use]
extern crate failure;

mod contract;
mod telegram_client;

use clap::{App, Arg};
use contract::Update;
use futures::future;
use futures::future::Either;
use futures::Stream;
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
use serde_json::from_slice;
use std::net::SocketAddr;
use std::sync::Arc;
use telegram_client::TelegramClient;

const STORAGE_DIR_NAME: &str = "storage";

fn main() {
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
    let telegram_client = TelegramClient::new(bot_token.into());
    let telegram_client = Arc::new(telegram_client);

    let server = Server::bind(&addr)
        .serve(move || {
            let telegram_client = telegram_client.clone();
            service_fn(move |x| echo(x, telegram_client.clone()))
        })
        .map_err(|e| error!("server error: {}", e));

    info!("Listening on http://{}", addr);
    rt::run(server);
}

fn echo(
    req: Request<Body>,
    telegram_client: Arc<TelegramClient>,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> + Send {
    let result = req.into_body().concat2().and_then(move |chunk| {
        let result = from_slice::<Update>(chunk.as_ref());
        match result {
            Ok(u) => {
                let chat_id = u.message.chat.id;
                Either::A(
                    telegram_client
                        .send_message(chat_id, "Hello from bot")
                        .then(move |result| {
                            let result = match result {
                                Ok(response) => {
                                    let is_success = response.status().is_success();
                                    if is_success {
                                        Either::A(future::ok(Response::new(Body::empty())))
                                    } else {
                                        Either::B(response.into_body().concat2().map(|chunk| {
                                            let bytes = chunk.into_bytes();
                                            let text: String = String::from_utf8_lossy(&bytes).into_owned();
                                            error!("{}", text);
                                            Response::builder()
                                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                .body(text.into())
                                                .unwrap()
                                        }))
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Unknown error {}",
                                        e.into_cause()
                                            .map(|c| c.description().to_string())
                                            .unwrap_or("".to_string())
                                    );
                                    Either::A(future::ok(
                                        Response::builder()
                                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                                            .body(Body::empty())
                                            .unwrap(),
                                    ))
                                }
                            };
                            result
                        }),
                )
            }
            Err(_) => Either::B(future::ok(
                Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(Body::empty())
                    .unwrap(),
            )),
        }
    });
    result
}
