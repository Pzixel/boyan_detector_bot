use hyper;
use hyper::{Client,Request,Method};
use hyper::client::{HttpConnector};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde;

pub struct TelegramClient<'a> {
    token: &'a String,
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
    core: Core
}

impl<'a> TelegramClient<'a> {
    pub fn new(token: &'a String) -> TelegramClient<'a> {
        let core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);
        TelegramClient{token, client, core}
    }

    pub fn send_message(&mut self, chat_id: i64, message: &str) -> serde::export::Result<hyper::Response, hyper::Error>{
        let url = format!("https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, message);
        let uri = url.parse().unwrap();
        let request = self.client.request(Request::new(Method::Post, uri));
        self.core.run(request)
    }
}