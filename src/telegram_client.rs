use bytes::Buf;
use bytes::Bytes;
use contract::*;
use futures::future;
use futures::Future;
use futures::Stream;
use hyper;
use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::from_slice;
use serde_json::Error as SerdeError;
use url::form_urlencoded::byte_serialize;

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
        Self { token, client }
    }

    pub fn get_me(&self) -> impl Future<Item = User, Error = TelegramClientError> {
        let url = format!("bot{}/getMe", self.token);
        let result = self
            .send(Method::GET, &url)
            .map_err(|e| TelegramClientError::HyperError(e))
            .and_then(|res| {
                res.into_body().into_future().then(|result| {
                    let (item, _) = result.map_err(|(e, _)| TelegramClientError::HyperError(e))?;
                    let chunk = item.unwrap();
                    let text: String = String::from_utf8_lossy(&chunk.bytes()).into_owned();
                    let result: ApiResult<User> =
                        from_slice(chunk.as_ref()).map_err(|e| TelegramClientError::SerdeError(e))?;
                    Ok(result.result)
                })
            });
        result
    }

    pub fn send_message(&self, chat_id: i64, text: &str) -> ResponseFuture {
        let text: String = byte_serialize(text.as_bytes()).collect();
        let url = format!("bot{}/sendMessage?chat_id={}&text={}", self.token, chat_id, text);
        self.send(Method::POST, &url)
    }

    pub fn get_file(&mut self, file_id: &str) -> impl Future<Item = File, Error = TelegramClientError> {
        let url = format!("bot{}/getFile?file_id={}", self.token, file_id);
        let result = self
            .send(Method::GET, &url)
            .map_err(|e| TelegramClientError::HyperError(e))
            .and_then(|res| {
                res.into_body().into_future().then(|result| {
                    let (item, _) = result.map_err(|(e, _)| TelegramClientError::HyperError(e))?;
                    let chunk = item.unwrap();
                    from_slice(chunk.as_ref()).map_err(|e| TelegramClientError::SerdeError(e))
                })
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
        let request = Request::builder().method(method).uri(uri).body(Body::empty()).unwrap();
        self.client.request(request)
    }
}
