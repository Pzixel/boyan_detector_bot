extern crate iron;
extern crate bodyparser;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate config;

use std::env;
use std::fmt;
mod contract;
use iron::prelude::*;
use contract::*;
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

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
            let url = format!("https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}", bot_token, chat_id, "Hello from bot!");
            let mut core = Core::new().unwrap();
            let handle = core.handle();
            let client = Client::configure()
                .connector(HttpsConnector::new(4, &handle).unwrap())
                .build(&handle);
            let request = client.request(hyper::Request::new(hyper::Method::Get, url.parse().unwrap()));
            core.run(request).unwrap();
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