use crate::ApiResult;
use crate::response::Response;
use reqwest::RequestBuilder;
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
    async fn dispatch(self) -> ApiResult<Vec<u8>>;
    async fn dispatch_empty(self) -> ApiResult<()>;
    async fn dispatch_text(self) -> ApiResult<String>;
    async fn dispatch_json<R: DeserializeOwned>(self) -> ApiResult<R>;
}

impl Request for RequestBuilder {
    async fn dispatch(self) -> ApiResult<Vec<u8>> {
        let response = self
            .send()
            .await
            .map_err(|e| Response::new().bad_request(e.to_string()))?;
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
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Response::new().bad_request(e.to_string()))?;

        let response = Response::new()
            .with_headers(headers)
            .with_body(bytes.to_vec());
        if status.is_client_error() || status.is_server_error() {
            Err(response.error(status, format!("request failed with {status}")))
        } else {
            Ok(response.with_status(status))
        }
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
