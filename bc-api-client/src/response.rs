use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

pub type ApiResult<T> = std::result::Result<Response<T>, Response<GenericError>>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GenericError {
    pub error: String,
    pub message: String,
}

impl GenericError {
    #[must_use]
    pub fn new(error: String) -> Self {
        Self {
            error,
            message: String::new(),
        }
    }

    #[must_use]
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

fn to_lossy(array: &[u8]) -> String {
    String::from_utf8_lossy(array).to_string()
}

#[derive(Clone, Debug)]
pub struct Response<R> {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: R,
}

impl Response<Vec<u8>> {
    /// Creates a new [Response] instance.
    ///
    /// # Errors
    ///
    /// Throws an error if the response status contains an error code.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(status: StatusCode, header_map: HeaderMap, bytes: Vec<u8>) -> ApiResult<Vec<u8>> {
        let headers = header_map
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();

        // check if returned status is error
        if status.is_client_error() || status.is_server_error() {
            Err(Response {
                status,
                headers,
                body: GenericError::new("status is error".to_string())
                    .with_message(to_lossy(&bytes)),
            })
        } else {
            Ok(Response {
                status,
                headers,
                body: bytes,
            })
        }
    }

    #[must_use]
    pub fn text(self) -> Response<String> {
        Response {
            status: self.status,
            headers: self.headers,
            body: to_lossy(&self.body),
        }
    }
    /// Attempts to deserialize the body of a [`reqwest::Response`] into the expected type.
    ///
    /// # Errors
    ///
    /// Returns an error if the response status is error or if deserialization fails.
    pub fn json<R: DeserializeOwned>(self) -> ApiResult<R> {
        let body = serde_json::from_slice(&self.body).map_err(|e| Response {
            status: StatusCode::BAD_REQUEST,
            headers: self.headers.clone(),
            body: GenericError::new(e.to_string()).with_message(to_lossy(&self.body)),
        })?;

        Ok(Response {
            status: self.status,
            headers: self.headers,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::Value;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestData {
        foo: u64,
        bar: String,
        baz: Option<u8>,
    }

    #[test]
    fn process_empty() {
        let status = StatusCode::OK;
        let mut header_map = HeaderMap::new();
        header_map.insert("foo", "bar".parse().unwrap());

        let response = Response::new(status, header_map, vec![]).unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
    }

        #[test]
        async fn process_string() {
            let input = "hello world";
            let test = TestResponse::new(StatusCode::OK, json!(input)).into_inner();
            let response = Response::<String>::decode(test).await.unwrap();
            assert_eq!(response.status, StatusCode::OK);
            assert_eq!(response.body, input);

            let input = "hello \nmy\" world";
            let test = TestResponse::new(StatusCode::ACCEPTED, json!(input)).into_inner();
            let response = Response::<String>::decode(test).await.unwrap();
            assert_eq!(response.status, StatusCode::ACCEPTED);
            assert_eq!(response.body, input);

            let input = "hello \nmy\" world";
            let test = TestResponse::raw(StatusCode::ACCEPTED, input.to_string()).into_inner();
            let response = Response::<String>::decode(test).await.unwrap();
            assert_eq!(response.status, StatusCode::ACCEPTED);
            assert_eq!(response.body, input);
        }

        /*
        #[tokio::test]
        async fn process_numeric() {
            let input = 1234;
            let test = TestResponse::new(StatusCode::ACCEPTED, json!(input)).into_inner();
            let response = Response::<u16>::decode(test).await.unwrap();
            assert_eq!(response.status, StatusCode::ACCEPTED);
            assert_eq!(response.body, input);

            let input = vec![1, 2, 3, 4];
            let test = TestResponse::new(StatusCode::ACCEPTED, json!(input)).into_inner();
            let response = Response::<Vec<u8>>::decode(test).await.unwrap();
            assert_eq!(response.status, StatusCode::ACCEPTED);
            assert_eq!(response.body, input);
        }

        #[tokio::test]
        async fn process_valid_body() {
            let input = json!({ "foo": 12, "bar": "mybar" });
            let test = TestResponse::new(StatusCode::CREATED, input).into_inner();
            let response = Response::<TestData>::decode(test).await.unwrap();
            assert_eq!(response.status, StatusCode::CREATED);
            assert_eq!(
                response.body,
                TestData {
                    foo: 12,
                    bar: "mybar".to_string(),
                    baz: None
                }
            );
        }

        #[tokio::test]
        async fn process_invalid_body() {
            let input = json!({ "foo": "12", "bar": "mybar" });
            let test = TestResponse::new(StatusCode::CREATED, input).into_inner();
            let response = Response::<TestData>::decode(test).await.unwrap_err();
            assert_eq!(response.status, StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn process_error() {
            let input = "invalid";
            let test = TestResponse::raw(StatusCode::IM_A_TEAPOT, input.to_string()).into_inner();
            let response = Response::<u64>::decode(test).await.unwrap_err();
            assert_eq!(response.status, StatusCode::IM_A_TEAPOT);
            assert_eq!(
                response.body,
                GenericError::new("status is error".to_string()).with_message(input.to_string())
            );
        }
    */
}
