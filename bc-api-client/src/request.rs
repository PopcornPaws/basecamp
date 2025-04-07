use crate::response::{ApiResult, GenericError, Response};
use reqwest::{RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;

use std::collections::HashMap;

#[allow(async_fn_in_trait)]
pub trait Request {
    /// Dispatches an API call and attempts to deserialize the response payload to a generic type
    /// `R` that is specified at compile time.
    ///
    /// Returns an error if the http request fails or the returned response body cannot be
    /// deserialized into the expected type.
    ///
    /// # Errors
    ///
    /// Throws an error if the request fails to send or if the response cannot be processed.
    async fn dispatch(self) -> ApiResult<Vec<u8>>;
    async fn dispatch_empty(self) -> ApiResult<()>;
    async fn dispatch_text(self) -> ApiResult<String>;
    async fn dispatch_json<R: DeserializeOwned>(self) -> ApiResult<R>;
}

impl Request for RequestBuilder {
    async fn dispatch(self) -> ApiResult<Vec<u8>> {
        let response = self.send().await.map_err(|e| Response {
            status: e.status().unwrap_or(StatusCode::BAD_REQUEST),
            headers: HashMap::new(),
            body: GenericError::new(e.to_string()),
        })?;
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.bytes().await.map_err(|e| Response {
            status: e.status().unwrap_or(StatusCode::BAD_REQUEST),
            headers: HashMap::new(),
            body: GenericError::new(e.to_string()),
        })?;

        Response::<Vec<u8>>::new(status, headers, bytes.to_vec())
    }

    async fn dispatch_empty(self) -> ApiResult<()> {
        Ok(self.dispatch().await?.empty())
    }

    async fn dispatch_text(self) -> ApiResult<String> {
        Ok(self.dispatch().await?.text())
    }

    async fn dispatch_json<R: DeserializeOwned>(self) -> ApiResult<R> {
        self.dispatch().await?.json()
    }
}
