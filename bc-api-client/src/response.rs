use crate::ApiResult;
use crate::error::GenericError;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Response<R> {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: R,
}

impl<R: Default> Default for Response<R> {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: R::default(),
        }
    }
}

impl Response<Vec<u8>> {
    #[must_use]
    pub fn empty(self) -> Response<()> {
        self.with_body(())
    }

    #[must_use]
    pub fn text(self) -> Response<String> {
        let body = self.body_to_utf8();
        self.with_body(body)
    }

    #[must_use]
    pub fn body_to_utf8(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
    /// Attempts to deserialize the body of a [`reqwest::Response`] into the expected type.
    ///
    /// # Errors
    ///
    /// Returns an error if the response status is error or if deserialization fails.
    pub fn json<R: DeserializeOwned>(self) -> ApiResult<R> {
        match serde_json::from_slice(&self.body) {
            Ok(body) => Ok(self.with_body(body)),
            Err(error) => Err(self.text().bad_request(error.to_string())),
        }
    }
}

impl Response<()> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<R> Response<R> {
    #[must_use]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    #[must_use]
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
    #[must_use]
    pub fn with_body<B>(self, body: B) -> Response<B> {
        Response {
            status: self.status,
            headers: self.headers,
            body,
        }
    }

    pub fn is_error(&self) -> bool {
        self.status.is_client_error() || self.status.is_server_error()
    }
}

impl<R: std::fmt::Debug> Response<R> {
    #[must_use]
    pub fn bad_request<E: ToString>(self, error: E) -> Response<GenericError> {
        self.error(StatusCode::BAD_REQUEST, error)
    }

    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn error<E: ToString>(self, status: StatusCode, error: E) -> Response<GenericError> {
        let message = format!("{:#?}", self.body);
        let response = self
            .with_status(status)
            .with_body(GenericError::new(error.to_string()).with_message(message));
        debug_assert!(response.is_error());
        response
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;
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
        let mut headers = HashMap::new();
        headers.insert("foo".to_string(), "bar".to_string());

        let response = Response::new().with_headers(headers);
        assert_eq!(response.status, status);
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
        assert_eq!(response.body, ());
    }

    #[test]
    fn process_string() {
        let input = "hello world";
        let response = Response::new().with_body(input);
        assert_eq!(response.body, input);

        let input = "hello \nmy\" world";
        let status = StatusCode::ACCEPTED;
        let response = Response::new()
            .with_status(status)
            .with_body(input.as_bytes().to_vec())
            .text();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = "hello \nmy\" world";
        let input_json_bytes = json!(input).to_string().as_bytes().to_vec();
        let response = Response::new()
            .with_body(input_json_bytes)
            .json::<String>()
            .unwrap();
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_numeric() {
        let status = StatusCode::NO_CONTENT;
        let input = 1234;
        let input_json_bytes = json!(input).to_string().as_bytes().to_vec();
        let response = Response::new()
            .with_status(status)
            .with_body(input_json_bytes)
            .json::<u16>()
            .unwrap();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = vec![1, 2, 3, 4];
        let input_json_bytes = json!(input).to_string().as_bytes().to_vec();
        let response = Response::new()
            .with_body(input_json_bytes)
            .json::<Vec<u64>>()
            .unwrap();
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_valid_body() {
        let input_json_bytes = json!({ "foo": 12, "bar": "mybar" })
            .to_string()
            .as_bytes()
            .to_vec();
        let response = Response::new()
            .with_body(input_json_bytes)
            .json::<TestData>()
            .unwrap();
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
        let input_json_bytes = json!({ "foo": "12", "bar": "mybar" })
            .to_string()
            .as_bytes()
            .to_vec();
        let response = Response::new()
            .with_body(input_json_bytes)
            .json::<TestData>()
            .unwrap_err();
        assert_eq!(response.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn process_error() {
        let status = StatusCode::IM_A_TEAPOT;

        let error = format!("request failed with status {status}");
        let response = Response::new().error(status, &error);
        assert!(response.is_error());
        assert_eq!(response.status, status);
        assert_eq!(response.body, GenericError::new(error).with_message("()"));
    }
}
