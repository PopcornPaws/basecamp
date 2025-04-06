use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

pub type ApiResult<T> = std::result::Result<Response<T>, Response<GenericError>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl<R: DeserializeOwned> TryFrom<reqwest::Response> for Response<R> {
    type Error = Response<GenericError>;
    fn try_from(response: reqwest::Response) -> ApiResult<R> {
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

        let message = futures::executor::block_on(async move {
            response.text().await.unwrap_or("null".to_string())
        });

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
            Err(error) => Err(Response {
                status: StatusCode::BAD_REQUEST,
                headers,
                body: GenericError::new(error.to_string()).with_message(message),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use reqwest::response::Response;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestData {
        foo: u64,
        bar: String,
        baz: Option<u8>,
    }

    #[test]
    fn process_empty() {
        let response = Response::<()>::try_from((StatusCode::OK, String::new())).unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    /*
    #[test]
    fn process_string() {
        let response =
            Response::<String>::try_from((StatusCode::ACCEPTED, "\"hello world\"".to_string()))
                .unwrap();
        assert_eq!(response.status, StatusCode::ACCEPTED);
        assert_eq!(response.body, "hello world");

        let response =
            Response::<String>::try_from((StatusCode::ACCEPTED, "hello world".to_string()))
                .unwrap();
        assert_eq!(response.status, StatusCode::ACCEPTED);
        assert_eq!(response.body, "hello world");

        let input = "hello \nmy\" world";
        let response =
            Response::<String>::try_from((StatusCode::ACCEPTED, input.to_string())).unwrap();
        assert_eq!(response.status, StatusCode::ACCEPTED);
        assert_eq!(response.body, input);
    }

    #[test]
    fn process_numeric() {
        let response = Response::<u16>::try_from((StatusCode::ACCEPTED, 1234.to_string())).unwrap();
        assert_eq!(response.status, StatusCode::ACCEPTED);
        assert_eq!(response.body, 1234);
    }

    #[test]
    fn process_valid_body() {
        let response = Response::<TestData>::try_from((
            StatusCode::CREATED,
            json!({ "foo": 12, "bar": "mybar" }).to_string(),
        ))
        .unwrap();
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

    #[test]
    fn process_invalid_body() {
        let response =
            Response::<TestData>::try_from((StatusCode::CREATED, "invalid stuff".to_string()))
                .unwrap_err();
        assert_eq!(response.status, StatusCode::BAD_REQUEST);
        assert_eq!(response.body, "invalid stuff");
    }

    #[test]
    fn process_error() {
        let response = Response::<u64>::try_from((
            StatusCode::FORBIDDEN,
            json!({ "message": "invalid credentials" }).to_string(),
        ))
        .unwrap_err();
        assert_eq!(response.status, StatusCode::FORBIDDEN);
        assert_eq!(response.body, "invalid credentials");

        let response = Response::<u64>::try_from((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "message": "database error"}).to_string(),
        ))
        .unwrap_err();
        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.body, "database error");
    }
*/
}
