extern crate bytes;
extern crate clap;
extern crate futures;
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
extern crate imagedb;
extern crate tokio;

mod contract;
mod telegram_client;

use clap::{App, Arg};
use contract::Update;
use futures::future;
use futures::future::Either;
use futures::IntoFuture;
use futures::Stream;
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
use imagedb::*;
use serde_json::from_slice;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use telegram_client::*;
use tokio::runtime::Runtime;

const STORAGE_DIR_NAME: &str = "storage";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ImageMetadata {
    file_name: String,
    user_id: i64,
    message_id: i64,
}

impl ImageMetadata {
    pub fn new(file_name: String, user_id: i64, message_id: i64) -> Self {
        Self {
            file_name,
            user_id,
            message_id,
        }
    }
}

impl Metadata for ImageMetadata {
    fn file_name(&self) -> &str {
        &self.file_name
    }
}

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
    let storage = FileStorage::<ImageMetadata>::new(STORAGE_DIR_NAME.into());
    let db = ImageDb::new(storage);
    let db = Arc::new(Mutex::new(db));

    let server = Server::bind(&addr)
        .serve(move || {
            let telegram_client = telegram_client.clone();
            let db = db.clone();

            service_fn(move |x| handle_request(x, telegram_client.clone(), db.clone()))
        })
        .map_err(|e| error!("server error: {}", e));

    info!("Listening on http://{}", addr);
    rt::run(server);
}

fn handle_request(
    req: Request<Body>,
    telegram_client: Arc<TelegramClient>,
    db: Arc<Mutex<ImageDb<ImageMetadata, FileStorage<ImageMetadata>>>>,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> + Send {
    req.into_body().concat2().and_then(move |chunk| {
        from_slice::<Update>(chunk.as_ref())
            .into_future()
            .map_err(|_| Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(Body::empty())
                .expect("known safe response parameters"))
            .and_then(|update| {
                let chat_id = update.message.chat.id;
                let message_id = update.message.message_id;
                let processing_info = match (&update.message.from.map(|x| x.id), &update.message.document, &update.message.photo) {
                    (Some(ref from), Some(ref document), _) => Some((from, &document.file_id)),
                    (Some(ref from), _, Some(ref photo)) => photo
                        .iter()
                        .max_by_key(|x| x.file_size.unwrap_or(0))
                        .map(|x| (from, &x.file_id)),
                    _ => None,
                };
                processing_info
                    .map(move |(user_id, file_id)| (user_id.clone(), file_id.clone(), chat_id, message_id))
                    .ok_or_else(|| {
                        info!("There is no sender or images. Skipping");
                        Response::new(Body::empty())
                    })
            })
            .and_then(move |(user_id, file_id, chat_id, message_id)| {
                telegram_client.get_file(&file_id)
                    .and_then(move |file| {
                        info!("Checking file {:?}", file);

                        if let Some((file_path, ext)) = get_file_path_if_processable(file.file_path) {
                            Either::A(telegram_client.download_file(&file_path).and_then(move |bytes| {
                                let image = Image::new(bytes.into_iter().collect(),
                                                       ImageMetadata::new(format!("{}.{}", file_id, ext), user_id, message_id));

                                let mut db = db.lock().unwrap();

                                if let ImageVariant::AlreadyExists(metadata) = db.save_image_if_new(image) {
                                    Either::A(telegram_client.send_message(
                                        chat_id,
                                        &format!(
                                            "Похоже, что [братишка](tg://user?id={}) боян добавил. Линк на оригинал выше.",
                                            metadata.user_id
                                        ),
                                        Some(metadata.message_id),
                                    ))
                                }
                                else {
                                    info!("New image! Congrats, user {}", user_id);
                                    Either::B(future::ok(()))
                                }
                            }))
                        } else {
                            info!("Skipping unsupported extension");
                            Either::B(future::ok(()))
                        }
                    })
                    .map_err(|e| {
                        error!("Error while processing: {}", e);
                        Response::builder()
                            .status(StatusCode::GATEWAY_TIMEOUT)
                            .body(Body::empty())
                            .expect("known safe response parameters")
                    })
            })
            // ...and here we unify both paths
            .map(|_| Response::new(Body::empty()))
            .or_else(Ok)
    })
}

fn get_file_path_if_processable(file_path: Option<String>) -> Option<(String, String)> {
    if let Some(file_path) = file_path {
        if let Some(ext) = file_path.rsplit('.').next().map(|x| x.to_string()) {
            let ext = ext.to_lowercase();
            if ext == "png" || ext == "jpg" || ext == "jpeg" {
                return Some((file_path, ext));
            }
        }
    }
    None
}
