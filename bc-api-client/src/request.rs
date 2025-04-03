use crate::response::Response;
use crate::Error;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

#[allow(async_fn_in_trait)]
pub trait Request {
    /// Dispatches an API call and extracts the status and response as string.
    ///
    /// # Errors
    ///
    /// Throws an error if the request fails to send or if the response cannot be processed.
    async fn dispatch<R: DeserializeOwned>(self) -> Result<Response, Error>;
}

impl Request for RequestBuilder {
    async fn dispatch<R: DeserializeOwned>(self) -> Result<Response, Error> {
        let response = self.send().await?;

        Ok(Response {
            status: response.status(),
            body: response.text().await?,
        })
    }
}
