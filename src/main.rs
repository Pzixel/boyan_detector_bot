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
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate failure;
extern crate tokio;

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
use telegram_client::*;
use tokio::runtime::Runtime;

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
        .arg(
            Arg::with_name("externalAddress")
                .short("e")
                .long("externalAddress")
                .help("Sets the external address where webhook should be setted up")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let bot_token = matches.value_of("token").unwrap();
    let address = matches.value_of("address").unwrap();
    let external_address = matches.value_of("externalAddress").unwrap();
    run(bot_token, address, external_address);
}

fn run(bot_token: &str, listening_address: &str, external_address: &str) {
    let addr: SocketAddr = listening_address.parse().unwrap();
    let telegram_client = TelegramClient::new(bot_token.into());

    let mut runtime = Runtime::new().unwrap();
    let me = runtime.block_on(telegram_client.get_me()).unwrap();

    info!("Started as {}", me.first_name);

    let web_hook_is_set = runtime
        .block_on(telegram_client.set_web_hook(external_address))
        .unwrap();

    if !web_hook_is_set {
        panic!("Couldn't set web hook. Cannot process updates.");
    }

    info!("Webhook has been set on {}", external_address);

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
            Ok(update) => {
                let chat_id = update.message.chat.id;
                let message_id = update.message.message_id;
                let file_id = match (&update.message.document, &update.message.photo) {
                    (Some(ref document), _) => Some(&document.file_id),
                    (_, Some(ref photo)) => photo
                        .iter()
                        .max_by_key(|x| x.file_size.unwrap_or(0))
                        .map(|x| &x.file_id),
                    _ => None,
                };

                match file_id {
                    Some(file_id) => Either::A({
                        let f = telegram_client.get_file(file_id).and_then(move |file| {
                            info!("Checking file {:?}", file);

                            let file_path = file.file_path.clone();

                            let inner_f = if let Some(file_path) = file_path {
                                Either::A(telegram_client.download_file(&file_path).and_then(move |bytes| {
                                    telegram_client.send_message(
                                        chat_id,
                                        &format!(
                                            "Hello from bot. Got file with id: {}. File length is {} bytes",
                                            file.file_id,
                                            bytes.len()
                                        ),
                                        Some(message_id),
                                    )
                                }))
                            } else {
                                Either::B(future::ok(()))
                            };
                            inner_f
                        });
                        f.then(move |result| {
                            let result = match result {
                                Ok(_) => Either::A(future::ok(Response::new(Body::empty()))),
                                Err(e) => {
                                    error!("Error while processing: {}", e);
                                    Either::B(future::ok(
                                        Response::builder()
                                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                                            .body(Body::empty())
                                            .unwrap(),
                                    ))
                                }
                            };
                            result
                        })
                    }),
                    None => Either::B(future::ok(Response::new(Body::empty()))),
                }
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
