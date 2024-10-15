use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub type Result<T> = std::result::Result<Response<T>, Response<String>>;

#[derive(Clone, Debug, Deserialize)]
pub struct GenericError {
    #[serde(alias = "msg")]
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct Response<R> {
    pub status: StatusCode,
    pub body: R,
}

impl<R: DeserializeOwned> TryFrom<(StatusCode, String)> for Response<R> {
    type Error = Response<String>;
    fn try_from((status, mut message): (StatusCode, String)) -> Result<R> {
        // create valid json from empty response by returning 'null'
        if message.is_empty() {
            message.push_str("null");
        }
        // check if returned status is error
        if status.is_client_error() || status.is_server_error() {
            if let Ok(error) = serde_json::from_str::<GenericError>(&message) {
                message = error.message;
            }
            Err(Response {
                status,
                body: message,
            })
        // try to deserialize response body into the expected type
        } else if let Ok(body) = serde_json::from_str::<R>(&message) {
            Ok(Response { status, body })
        // try to deserialize response body into the expected type by casting it to a valid json
        // string
        } else if let Ok(body) = serde_json::from_str::<R>(&format!("\"{message}\"")) {
            Ok(Response { status, body })
        } else {
            Err(Response {
                status: StatusCode::BAD_REQUEST,
                body: message,
            })
        }
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
    fn process_empty() {
        let response = Response::<()>::try_from((StatusCode::OK, String::new())).unwrap();
        assert_eq!(response.status, StatusCode::OK);
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
