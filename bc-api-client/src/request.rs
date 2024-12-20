use crate::response::{Response, Result};
use reqwest::{RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;

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
    async fn dispatch<R: DeserializeOwned>(self) -> Result<R>;
}

impl Request for RequestBuilder {
    async fn dispatch<R: DeserializeOwned>(self) -> Result<R> {
        let response = self.send().await.map_err(|e| Response {
            status: e.status().unwrap_or(StatusCode::BAD_REQUEST),
            body: e.to_string(),
        })?;
        let status = response.status();
        let message = response.text().await.map_err(|e| Response {
            status: e.status().unwrap_or(StatusCode::BAD_REQUEST),
            body: e.to_string(),
        })?;
        (status, message).try_into()
    }
}
