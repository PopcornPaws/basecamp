use reqwest::StatusCode;
use reqwest::header::HeaderMap;
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
    pub fn empty(self) -> Response<()> {
        Response {
            status: self.status,
            headers: self.headers,
            body: (),
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
    use serde_json::json;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestData {
        foo: u64,
        bar: String,
        baz: Option<u8>,
    }

    #[test]
    #[allow(clippy::unit_cmp)]
    fn process_empty() {
        let status = StatusCode::OK;
        let mut header_map = HeaderMap::new();
        header_map.insert("foo", "bar".parse().unwrap());

        let response = Response::new(status, header_map, vec![]).unwrap();
        assert_eq!(response.status, status);
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
        assert!(response.body.is_empty());

        let response = response.empty();
        assert_eq!(response.status, status);
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
        assert_eq!(response.body, ());
    }

    #[test]
    fn process_string() {
        let input = "hello world";
        let status = StatusCode::OK;
        let header_map = HeaderMap::new();

        let response = Response::new(status, header_map.clone(), input.as_bytes().to_vec())
            .unwrap()
            .text();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = "hello \nmy\" world";
        let status = StatusCode::ACCEPTED;
        let response = Response::new(status, header_map.clone(), input.as_bytes().to_vec())
            .unwrap()
            .text();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = "hello \nmy\" world";
        let input_json = json!("hello \nmy\" world").to_string();
        let response = Response::new(status, header_map, input_json.as_bytes().to_vec())
            .unwrap()
            .json::<String>()
            .unwrap();
        assert_eq!(response.status, StatusCode::ACCEPTED);
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_numeric() {
        let status = StatusCode::NO_CONTENT;
        let header_map = HeaderMap::new();

        let input = 1234;
        let input_json = json!(input).to_string();
        let response = Response::new(status, header_map.clone(), input_json.as_bytes().to_vec())
            .unwrap()
            .json::<u16>()
            .unwrap();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = vec![1, 2, 3, 4];
        let input_json = json!(input).to_string();
        let response = Response::new(status, header_map, input_json.as_bytes().to_vec())
            .unwrap()
            .json::<Vec<u64>>()
            .unwrap();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_valid_body() {
        let status = StatusCode::NO_CONTENT;
        let header_map = HeaderMap::new();
        let input_json = json!({ "foo": 12, "bar": "mybar" }).to_string();
        let response = Response::new(status, header_map, input_json.as_bytes().to_vec())
            .unwrap()
            .json::<TestData>()
            .unwrap();
        assert_eq!(response.status, status);
        assert_eq!(
            response.body,
            TestData {
                foo: 12,
                bar: "mybar".to_string(),
                baz: None
            }
        );
    }

    #[test]
    fn process_invalid_body() {
        let status = StatusCode::OK;
        let header_map = HeaderMap::new();
        let input_json = json!({ "foo": "12", "bar": "mybar" }).to_string();
        let response = Response::new(status, header_map, input_json.as_bytes().to_vec())
            .unwrap()
            .json::<TestData>()
            .unwrap_err();
        assert_eq!(response.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn process_error() {
        let status = StatusCode::IM_A_TEAPOT;
        let header_map = HeaderMap::new();

        let input = "invalid";
        let response = Response::new(status, header_map, input.as_bytes().to_vec()).unwrap_err();
        assert_eq!(response.status, status);
        assert_eq!(
            response.body,
            GenericError::new("status is error".to_string()).with_message(input.to_string())
        );
    }
}
