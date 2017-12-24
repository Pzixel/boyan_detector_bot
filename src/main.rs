extern crate iron;
extern crate bodyparser;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate config;

mod contract;
mod telegram_client;

use std::env;
use std::fmt;
use iron::prelude::*;
use contract::*;
use telegram_client::*;

#[derive(Debug, Clone, Deserialize)]
struct Settings {
    address: String
}

fn main() {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("Settings")).unwrap();
    let settings = settings.try_into::<Settings>().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Expected bot token as parameter, but found {} parameters!", args.len() - 1);
    }

    let bot_token = args[1].clone();
    let chain = Chain::new(move |r: &mut Request| web_hook(r, &bot_token));
    Iron::new(chain).http(settings.address).unwrap();
}

fn web_hook(request: &mut Request, bot_token: &String) -> IronResult<Response> {
    let update = request.get::<bodyparser::Struct<Update>>();
    match update {
        Ok(Some(update)) => {
            let update: Update = update;
            let chat_id = update.message.chat.id;
            let mut client = TelegramClient::new(bot_token);
            client.send_message(chat_id, "Hello from bot!").unwrap();
            Ok(Response::with((iron::status::Ok)))
        },
        Ok(None) => {
            const COULD_NOT_PARSE_UPDATE_MESSAGE : &str = "Could not parse update object";
            Err(iron::IronError::new(BotError::new(COULD_NOT_PARSE_UPDATE_MESSAGE), (iron::status::InternalServerError, COULD_NOT_PARSE_UPDATE_MESSAGE)))
        },
        Err(err) => {
            let message = err.to_string();
            Err(iron::IronError::new(err, (iron::status::InternalServerError, message)))
        }
    }
}

#[derive(Debug)]
struct BotError<'a> {
    details: &'a str
}

impl<'a> BotError<'a> {
    fn new(msg: &'a str) -> BotError<'a> {
        BotError{details: msg}
    }
}

impl<'a> fmt::Display for BotError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl<'a> std::error::Error for BotError<'a> {
    fn description(&self) -> &str {
        self.details
    }
}