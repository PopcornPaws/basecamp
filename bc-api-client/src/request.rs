use crate::response::{Response, Result};
use reqwest::{RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;

use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Request(RequestBuilder);

impl Deref for Request {
    type Target = RequestBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<RequestBuilder> for Request {
    fn from(builder: RequestBuilder) -> Self {
        Self(builder)
    }
}

impl From<Request> for RequestBuilder {
    fn from(request: Request) -> Self {
        request.into_inner()
    }
}

impl Request {
    /// Dispatches an API call and attempts to deserialize the response payload to a generic type
    /// `R` that is specified at compile time.
    ///
    /// Returns an error if the http request fails or the returned response body cannot be
    /// deserialized into the expected type.
    ///
    /// # Errors
    ///
    /// Throws an error if the request fails to send or if the response cannot be processed.
    pub async fn dispatch<R: DeserializeOwned>(self) -> Result<R> {
        let response = self.0.send().await.map_err(|e| Response {
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

    pub fn into_inner(self) -> RequestBuilder {
        self.0
    }
}
