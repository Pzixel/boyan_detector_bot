extern crate iron;
extern crate bodyparser;
extern crate persistent;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use std::fmt;
mod contract;
use iron::prelude::*;
use contract::*;
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

const TOKEN: &str = "";

fn main() {
    let chain = Chain::new(web_hook);
    Iron::new(chain).http("127.0.0.1:62800").unwrap();
}

fn web_hook(request: &mut Request) -> IronResult<Response> {
    let update = request.get::<bodyparser::Struct<Update>>();
    match update {
        Ok(Some(update)) => {
            let update : Update = update;
            let chat_id = update.message.chat.id;
            let url = format!("https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}", TOKEN, chat_id, "Hello from bot!");
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