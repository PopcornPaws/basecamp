use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Clone, Debug)]
pub struct Response<R> {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: R,
}

impl<R: DeserializeOwned> Response<R> {
    /// Attempts to deserialize the body of a [`reqwest::Response`] into the expected type.
    ///
    /// # Errors
    ///
    /// Returns an error if the response status is error or if deserialization fails.
    pub async fn decode(response: reqwest::Response) -> ApiResult<R> {
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();

        let message: String = response.text().await.unwrap_or_default();

        Self::try_from((status, headers, message))
    }
}

impl<R: DeserializeOwned> TryFrom<(StatusCode, HashMap<String, String>, String)> for Response<R> {
    type Error = Response<GenericError>;
    fn try_from(
        (status, headers, mut message): (StatusCode, HashMap<String, String>, String),
    ) -> ApiResult<R> {
        if message.is_empty() {
            message.push_str("null");
        }

        // check if returned status is error
        if status.is_client_error() || status.is_server_error() {
            return Err(Response {
                status,
                headers,
                body: GenericError::new("status is error".to_string()).with_message(message),
            });
        }

        // try to deserialize response body into the expected type
        match serde_json::from_str::<R>(&message) {
            Ok(body) => Ok(Response {
                status,
                headers,
                body,
            }),
            Err(error) =>
            // as a fallback, try to cast response into valid json string
            {
                if let Ok(body) = serde_json::from_value::<R>(json!(message)) {
                    Ok(Response {
                        status,
                        headers,
                        body,
                    })
                } else {
                    Err(Response {
                        status: StatusCode::BAD_REQUEST,
                        headers,
                        body: GenericError::new(error.to_string()).with_message(message),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http::response::Builder;
    use serde_json::Value;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestData {
        foo: u64,
        bar: String,
        baz: Option<u8>,
    }

    struct TestResponse(reqwest::Response);

    impl TestResponse {
        #[allow(clippy::needless_pass_by_value)]
        pub fn new(status: StatusCode, body: Value) -> Self {
            Self::raw(status, body.to_string())
        }

        pub fn raw(status: StatusCode, body: String) -> Self {
            let res = reqwest::Response::from(
                Builder::new()
                    .status(status)
                    .header("foo", "bar")
                    .body(body)
                    .unwrap(),
            );
            Self(res)
        }

        pub fn into_inner(self) -> reqwest::Response {
            self.0
        }
    }

    #[tokio::test]
    async fn process_empty() {
        let test = TestResponse::new(StatusCode::OK, json!(())).into_inner();
        let response = Response::<()>::decode(test).await.unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
    }

    #[tokio::test]
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
}
