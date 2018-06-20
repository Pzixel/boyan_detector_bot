use contract::*;
use futures;
use futures::IntoFuture;
use futures::{Future, Stream};
use hyper;
use hyper::client::HttpConnector;
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde;
use serde_json;
use std;
use std::io;
use tokio_core::reactor::Core;

pub struct TelegramClient<'a> {
    token: &'a String,
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
    core: Core,
}

impl<'a> TelegramClient<'a> {
    pub fn new(token: &'a String) -> Self {
        let core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);
        TelegramClient { token, client, core }
    }

    pub fn send_message(&mut self, chat_id: i64, text: &str) -> serde::export::Result<hyper::Response, hyper::Error> {
        let url = format!("bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, text);
        self.send(Method::Post, &url)
    }

    pub fn get_file(&mut self, file_id: &str) -> serde::export::Result<File, hyper::Error> {
        let url = format!("bot{}/getFile?file_id={}", self.token, file_id);
        self.send_then(Method::Get, &url, |res| {
            res.body().concat2().and_then(move |body| {
                let v = body.to_vec();
                let response = String::from_utf8_lossy(&v).to_string();
                println!("Response = {}", response);
                let v: GetFileResponse =
                    serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(v.result)
            })
        })
    }

    pub fn download_file(&mut self, file_path: &str) -> serde::export::Result<Vec<u8>, hyper::Error> {
        let url = format!("file/bot{}/{}", self.token, file_path);
        self.send_then(Method::Get, &url, |res| {
            res.body().concat2().and_then(move |body| {
                let result = body.to_vec();
                Ok(result)
            })
        })
    }

    fn send(&mut self, method: Method, url: &str) -> serde::export::Result<hyper::Response, hyper::Error> {
        self.send_then(method, url, |x| Ok(x))
    }

    fn send_then<F, B>(
        &mut self,
        method: Method,
        url: &str,
        f: F,
    ) -> serde::export::Result<<B as futures::IntoFuture>::Item, hyper::Error>
    where
        F: std::ops::FnOnce(hyper::Response) -> B,
        B: IntoFuture<Error = hyper::Error>,
    {
        let uri = format!("https://api.telegram.org/{}", url).parse().unwrap();
        let request = self.client.request(Request::new(method, uri)).and_then(f);
        self.core.run(request)
    }
}
