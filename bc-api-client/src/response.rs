use crate::ApiResult;
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
    pub fn into_empty(self) -> Response<()> {
        self.with_body(())
    }

    #[must_use]
    pub fn into_text(self) -> Response<String> {
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
    pub fn try_into_json<R: DeserializeOwned>(self) -> ApiResult<R> {
        let body = serde_json::from_slice(&self.body)?;
        Ok(self.with_body(body))
    }
}

impl Response<()> {
    #[must_use]
    pub fn empty() -> Self {
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

        let response = Response::empty().with_headers(headers);
        assert_eq!(response.status, status);
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
        assert_eq!(response.body, ());
    }

    #[test]
    fn process_string() {
        let input = "hello world";
        let response = Response::empty().with_body(input);
        assert_eq!(response.body, input);

        let input = "hello \nmy\" world";
        let status = StatusCode::ACCEPTED;
        let response = Response::empty()
            .with_status(status)
            .with_body(input.as_bytes().to_vec())
            .into_text();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = "hello \nmy\" world";
        let input_json_bytes = json!(input).to_string().as_bytes().to_vec();
        let response = Response::empty()
            .with_body(input_json_bytes)
            .try_into_json::<String>()
            .unwrap();
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_numeric() {
        let status = StatusCode::NO_CONTENT;
        let input = 1234;
        let input_json_bytes = json!(input).to_string().as_bytes().to_vec();
        let response = Response::empty()
            .with_status(status)
            .with_body(input_json_bytes)
            .try_into_json::<u16>()
            .unwrap();
        assert_eq!(response.status, status);
        assert_eq!(response.body, input);

        let input = vec![1, 2, 3, 4];
        let input_json_bytes = json!(input).to_string().as_bytes().to_vec();
        let response = Response::empty()
            .with_body(input_json_bytes)
            .try_into_json::<Vec<u64>>()
            .unwrap();
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_valid_json() {
        let input_json_bytes = json!({ "foo": 12, "bar": "mybar" })
            .to_string()
            .as_bytes()
            .to_vec();
        let response = Response::empty()
            .with_body(input_json_bytes)
            .try_into_json::<TestData>()
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
    fn process_invalid_json() {
        let input_json_bytes = json!({ "foo": "12", "bar": "mybar" })
            .to_string()
            .as_bytes()
            .to_vec();
        assert!(
            Response::empty()
                .with_body(input_json_bytes)
                .try_into_json::<TestData>()
                .is_err()
        );
    }

    #[test]
    fn process_error() {
        let status = StatusCode::IM_A_TEAPOT;

        let response = Response::empty().with_status(status);
        assert!(response.is_error());
    }
}
