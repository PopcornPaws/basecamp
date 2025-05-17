use crate::ApiResult;
use crate::response::Response;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

#[allow(async_fn_in_trait)]
pub trait Request {
    /// Dispatches an API call and attempts to decode the response body into a byte vector.
    ///
    /// # Errors
    ///
    /// Throws an error if the request fails to complete or the body cannot be decoded.
    async fn request(self) -> ApiResult<Vec<u8>>;
    async fn request_empty(self) -> ApiResult<()>;
    async fn request_text(self) -> ApiResult<String>;
    async fn request_json<R: DeserializeOwned>(self) -> ApiResult<R>;
}

impl Request for RequestBuilder {
    async fn request(self) -> ApiResult<Vec<u8>> {
        let response = self.send().await?;
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
        let bytes = response.bytes().await?;

        Ok(Response::empty()
            .with_status(status)
            .with_headers(headers)
            .with_body(bytes.to_vec()))
    }

    async fn request_empty(self) -> ApiResult<()> {
        Ok(self.request().await?.into_empty())
    }

    async fn request_text(self) -> ApiResult<String> {
        Ok(self.request().await?.into_text())
    }

    async fn request_json<R: DeserializeOwned>(self) -> ApiResult<R> {
        self.request().await?.try_into_json()
    }
}
