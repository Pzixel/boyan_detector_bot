use futures::Future;
use futures::Stream;
use http::Error;
use hyper;
use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
use hyper::Body;
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;

pub struct TelegramClient<'a> {
    token: &'a String,
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
}

impl<'a> TelegramClient<'a> {
    pub fn new(token: &'a String, handle: &()) -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client: Client<_, Body> = Client::builder().build(https);
        TelegramClient { token, client }
    }

    pub fn send_message(&self, chat_id: i64, text: &str) -> Result<ResponseFuture, Error> {
        let url = format!("bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, text);
        self.send(Method::POST, &url)
    }

    pub fn download_file(&mut self, file_path: &str) -> Result<Body, Error> {
        let url = format!("file/bot{}/{}", self.token, file_path);
        self.send(Method::GET, &url)?
            .map(|res| res.into_body().for_each(|x| x.into_bytes()))
    }

    fn send(&self, method: Method, url: &str) -> Result<ResponseFuture, Error> {
        let uri = format!("https://api.telegram.org/{}", url);
        let request = Request::post(uri).body(Body::empty())?;
        Ok(self.client.request(request))
    }
}
