extern crate bytes;
extern crate clap;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate log4rs;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod contract;
mod telegram_client;

use clap::{App, Arg};

const STORAGE_DIR_NAME: &str = "storage";

fn main() {
    log4rs::init_file("log4rs.toml", Default::default()).unwrap();
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
    println!("Hello {}, address is {}", bot_token, listening_address);
}
