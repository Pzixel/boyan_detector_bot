use bytes::Bytes;
use contract::*;
use future::Either;
use futures::future;
use futures::Future;
use futures::Stream;
use hyper;
use hyper::client::HttpConnector;
use hyper::client::ResponseFuture;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde_json::from_slice;
use serde_json::Error as SerdeError;

#[derive(Debug, Fail)]
/// Custom errors that may happen during calls
pub enum TelegramClientError {
    #[fail(display = "Hyper error: {:?}", _0)]
    HyperError(hyper::Error),
    #[fail(display = "Serde error: {:?}", _0)]
    SerdeError(SerdeError),
    #[fail(display = "Connection error: {:?}", _0)]
    ConnectionError(String),
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

    pub fn set_web_hook(&self, address: &str) -> impl Future<Item = bool, Error = TelegramClientError> {
        let url = format!("bot{}/setWebhook?url={}/update", self.token, address);
        self.send_and_deserialize::<ApiResult<bool>>(Method::POST, &url, Body::empty())
            .map(|result| result.result)
    }

    pub fn get_me(&self) -> impl Future<Item = User, Error = TelegramClientError> {
        let url = format!("bot{}/getMe", self.token);
        self.send_and_deserialize::<ApiResult<User>>(Method::GET, &url, Body::empty())
            .map(|result| result.result)
    }

    pub fn send_message(
        &self,
        chat_id: i64,
        text: &str,
        reply_to_message_id: Option<i64>,
    ) -> impl Future<Item = (), Error = TelegramClientError> {
        let url = format!("bot{}/sendMessage", self.token);
        let value = json!({
            "chat_id": chat_id,
            "text": text,
            "reply_to_message_id": reply_to_message_id
        });
        let json = value.to_string();
        self.send_and_deserialize(Method::POST, &url, json.into())
    }

    pub fn get_file(&self, file_id: &str) -> impl Future<Item = File, Error = TelegramClientError> {
        let url = format!("bot{}/getFile?file_id={}", self.token, file_id);
        self.send_and_deserialize(Method::GET, &url, Body::empty())
    }

    pub fn download_file(&self, file_path: &str) -> impl Future<Item = Bytes, Error = hyper::Error> {
        let url = format!("file/bot{}/{}", self.token, file_path);
        self.send(Method::GET, &url, Body::empty()).and_then(|res| {
            res.into_body()
                .into_future()
                .map(|(item, _)| item.unwrap().into_bytes())
                .map_err(|(err, _)| err)
        })
    }

    fn send_and_deserialize<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Body,
    ) -> impl Future<Item = T, Error = TelegramClientError> {
        let result = self
            .send(method, &url, body)
            .map_err(|e| TelegramClientError::HyperError(e))
            .then(|result| match result {
                Ok(response) => {
                    let is_success = response.status().is_success();
                    let result = response.into_body().concat2().then(move |result| {
                        let chunk = result.map_err(|e| TelegramClientError::HyperError(e))?;
                        if true {
                            from_slice(chunk.as_ref()).map_err(|e| TelegramClientError::SerdeError(e))
                        } else {
                            let bytes = chunk.into_bytes();
                            let text: String = String::from_utf8_lossy(&bytes).into_owned();
                            Err(TelegramClientError::ConnectionError(text))
                        }
                    });
                    Either::A(result)
                }
                Err(e) => Either::B(future::err(e)),
            });
        result
    }

    fn send(&self, method: Method, url: &str, body: Body) -> ResponseFuture {
        let uri = format!("https://api.telegram.org/{}", url);
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap();
        self.client.request(request)
    }
}
