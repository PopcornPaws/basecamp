use reqwest::StatusCode;
use serde::de::DeserializeOwned;

#[derive(Clone, Debug)]
pub struct Response {
    pub status: StatusCode,
    pub body: String,
}

impl Response {
    pub fn new(status: StatusCode, body: String) -> Self {
        Self { status, body }
    }
    /// Attempts to deserialize the body string as json.
    ///
    /// # Errors
    ///
    /// Throws an error if the body cannot be deserialized.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        let body = if self.body.is_empty() {
            "\"null\""
        } else {
            self.body.as_str()
        };

        serde_json::from_str(body)
    }
}

/*
impl<R: DeserializeOwned> TryFrom<(StatusCode, String)> for Response<R> {
    type Error = Response<String>;
    fn try_from((status, mut message): (StatusCode, String)) -> Result<R> {
        // create valid json from empty response by returning 'null'
        // check if returned status is error
        if status.is_client_error() || status.is_server_error() {
            Err(Response {
                status,
                body: message,
            })
        // try to deserialize response body into the expected type
        } else if let Ok(body) = serde_json::from_str::<R>(&message) {
            Ok(Response { status, body })
        // as a fallback, try to deserialize response body into a String by casting it to
        // a valid json string first
        // NOTE unwrap is fine, because we are always serializing a valid UTF-8 String type
        } else if let Ok(body) =
            serde_json::from_str::<R>(&serde_json::to_string(&message).unwrap())
        {
            Ok(Response { status, body })
        } else {
            Err(Response {
                status,
                body: message,
            })
        }
    }
}
*/

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
    fn process_empty() {
        let response = Response::new(StatusCode::OK, String::new());
        assert!(response.
    }

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
}
