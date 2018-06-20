use contract::*;
use futures;
use futures::IntoFuture;
use futures::{Future, Stream};
use hyper;
use hyper::client::HttpConnector;
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;
use std;
use std::io;
use serde::export;
use serde_json::from_slice;
use hyper::Response;

pub struct TelegramClient<'a> {
    token: &'a String,
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
}

impl<'a> TelegramClient<'a> {
    pub fn new(token: &'a String, handle: &()) -> Self {
        let client = Client::configure()
            .connector(HttpsConnector::new(4).unwrap())
            .build(&handle);
        TelegramClient { token, client, }
    }

    pub fn send_message(&mut self, chat_id: i64, text: &str) -> export::Result<Response<()>, hyper::Error> {
        let url = format!("bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, text);
        self.send(Method::Post, &url)
    }

    pub fn get_file(&mut self, file_id: &str) -> export::Result<File, hyper::Error> {
        let url = format!("bot{}/getFile?file_id={}", self.token, file_id);
        self.send_then(Method::Get, &url, |res| {
            res.body().concat2().and_then(move |body| {
                let v = body.to_vec();
                let response = String::from_utf8_lossy(&v).to_string();
                println!("Response = {}", response);
                let v: GetFileResponse =
                    from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(v.result)
            })
        })
    }

    pub fn download_file(&mut self, file_path: &str) -> export::Result<Vec<u8>, hyper::Error> {
        let url = format!("file/bot{}/{}", self.token, file_path);
        self.send_then(Method::Get, &url, |res| {
            res.body().concat2().and_then(move |body| {
                let result = body.to_vec();
                Ok(result)
            })
        })
    }

    fn send<T>(&mut self, method: Method, url: &str) -> export::Result<Response<T>, hyper::Error> {
        self.send_then(method, url, |x| Ok(x))
    }

    fn send_then<T, F, B>(
        &mut self,
        method: Method,
        url: &str,
        f: F,
    ) -> export::Result<<B as futures::IntoFuture>::Item, hyper::Error>
    where
        F: std::ops::FnOnce(Response<T>) -> B,
        B: IntoFuture<Error = hyper::Error>,
    {
        let uri = format!("https://api.telegram.org/{}", url).parse().unwrap();
        let request = self.client.request(Request::new(method, uri)).and_then(f);
        request
    }
}
