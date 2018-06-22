use bytes::Buf;
use bytes::Bytes;
use contract::File;
use futures::future;
use futures::Future;
use futures::Stream;
use hyper;
use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
use hyper::{Body, Chunk, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::from_slice;
use serde_json::Error as SerdeError;

#[derive(Debug, Fail)]
/// Custom errors that may happen during calls
pub enum TelegramClientError {
    #[fail(display = "Hyper error: {:?}", _0)]
    HyperError(hyper::Error),
    #[fail(display = "Serde error: {:?}", _0)]
    SerdeError(SerdeError),
}

pub struct TelegramClient {
    token: String,
    client: Client<HttpsConnector<HttpConnector>, hyper::Body>,
}

impl TelegramClient {
    pub fn new(token: String) -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let client: Client<_, Body> = Client::builder().build(https);
        TelegramClient { token, client }
    }

    pub fn send_message(&self, chat_id: i64, text: &str) -> ResponseFuture {
        let url = format!("bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, text);
        self.send(Method::POST, &url)
    }

    pub fn get_file(&mut self, file_id: &str) -> impl Future<Item = File, Error = TelegramClientError> {
        let url = format!("bot{}/getFile?file_id={}", self.token, file_id);
        let result = self
            .send(Method::GET, &url)
            .map_err(|e| TelegramClientError::HyperError(e))
            .and_then(|res| {
                let foo = res.into_body().into_future().then(|result| {
                    let (item, _) = result.map_err(|(e, _)| TelegramClientError::HyperError(e))?;
                    let chunk = item.unwrap();
                    let file: File = from_slice(chunk.bytes()).map_err(|e| TelegramClientError::SerdeError(e))?;
                    Ok(file)
                });
                foo
            });
        result
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
