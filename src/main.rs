extern crate bodyparser;
extern crate config;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate iron;
extern crate itertools;
extern crate log4rs;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate tokio_core;

mod contract;
mod telegram_client;

use std::env;
use std::fmt;
use iron::prelude::*;
use contract::*;
use telegram_client::*;
use std::borrow::{Borrow, Cow};
use std::error::Error as ErrorTrait;
use std::fs::File;
use sha2::{Digest, Sha256};
use itertools::Itertools;
use std::path::Path;
use std::io::Write;

const STORAGE_DIR_NAME: &str = "storage";

#[derive(Debug, Clone, Deserialize)]
struct Settings {
    address: String,
}

fn main() {
    log4rs::init_file("log4rs.toml", Default::default()).unwrap();
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("Settings")).unwrap();
    let settings = settings.try_into::<Settings>().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!(
            "Expected bot token as single parameter, but found {} parameters!",
            args.len() - 1
        );
    }
    std::fs::create_dir_all(STORAGE_DIR_NAME).unwrap();
    let bot_token = args[1].clone();

    debug!("Starting application with bot token {}", bot_token);
    let chain = Chain::new(move |r: &mut Request| web_hook(r, &bot_token));
    Iron::new(chain).http(settings.address).unwrap();
}

fn web_hook(request: &mut Request, bot_token: &String) -> IronResult<Response> {
    match core(request, bot_token) {
        Ok(_) => Ok(Response::with((iron::status::Ok))),
        Err(error) => {
            error!("Error while processing the request: {:?}", error);
            let message = String::from(error.description());
            Err(IronError {
                error,
                response: Response::with((iron::status::InternalServerError, message))
            })
        }
    }
}

fn core(request: &mut Request, bot_token: &String) -> Result<(), Box<ErrorTrait+Send+Sync>> {
    let update = request
        .get::<bodyparser::Struct<Update>>()?;
    match update {
        Some(update) => {
            let update: Update = update;
            let chat_id = update.message.chat.id;
            let mut client = TelegramClient::new(bot_token);
            client
                .send_message(chat_id, "Hello from bot!")?;

            if let Some(document) = update.message.document {
                handle_document(&mut client, &document.file_id)?;
            }
            if let Some(photos) = update.message.photo {
                debug!("Found some photos! {} items", photos.len());
                for photo in photos {
                    handle_document(&mut client, &photo.file_id)?;
                }
            }

            Ok(())
        }
        _ => {
            const COULD_NOT_PARSE_UPDATE_MESSAGE: &str = "Could not parse update object";
            Err(Box::new(BotError::new(COULD_NOT_PARSE_UPDATE_MESSAGE)).into())
        }
    }
}

fn handle_document(client: &mut TelegramClient, file_id: &str) -> Result<(), Box<ErrorTrait+Send+Sync>> {
    let file = client
        .get_file(file_id)?;
    match file.file_path {
        Some(file_path) => {
            let file_bytes = client
                .download_file(&file_path)?;
            let mut hasher = Sha256::default();
            hasher.input(&file_bytes);
            let output = hasher.result();
            let file_hash = format!("{:02x}", output.iter().format(""));
            let filename = Path::new(&file_hash);
            let filename = match Path::new(&file_path).extension() {
                Some(extension) => filename.with_extension(extension),
                _ => filename.to_path_buf(),
            };
            debug!(
                "Processing file: {}. Resulting path: {:?}",
                file_path, filename
            );
            let mut file = File::create(Path::new(STORAGE_DIR_NAME).join(filename))?;
            file.write_all(&file_bytes)?;
            Ok(())
        }
        _ => Err(Box::new(BotError::new("File contains no path!")).into()),
    }
}

#[derive(Debug)]
struct BotError<'a> {
    details: Cow<'a, str>,
}

impl<'a> BotError<'a> {
    fn new<S: Into<Cow<'a, str>>>(msg: S) -> BotError<'a> {
        BotError {
            details: msg.into(),
        }
    }
}

impl<'a> fmt::Display for BotError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl<'a> ErrorTrait for BotError<'a> {
    fn description(&self) -> &str {
        self.details.borrow()
    }
}
