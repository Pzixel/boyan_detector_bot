use bytes::Bytes;
use futures::Future;
use futures::Stream;
use hyper;
use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
use hyper::Body;
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;

pub struct TelegramClient {
    token: String,
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
}

impl TelegramClient {
    pub fn new(token: String, handle: &()) -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client: Client<_, Body> = Client::builder().build(https);
        TelegramClient { token, client }
    }

    pub fn send_message(&self, chat_id: i64, text: &str) -> ResponseFuture {
        let url = format!("bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, text);
        self.send(Method::POST, &url)
    }

    pub fn download_file(&mut self, file_path: &str) -> impl Future<Item = Bytes, Error = hyper::Error> {
        let url = format!("file/bot{}/{}", self.token, file_path);
        self.send(Method::GET, &url).and_then(|res| {
            res.into_body()
                .into_future()
                .map(|(item, _)| item.unwrap().into_bytes())
                .map_err(|(err, _)| err)
        })
    }

    fn send(&self, method: Method, url: &str) -> ResponseFuture {
        let uri = format!("https://api.telegram.org/{}", url);
        let request = Request::post(uri).body(Body::empty()).unwrap();
        self.client.request(request)
    }
}
