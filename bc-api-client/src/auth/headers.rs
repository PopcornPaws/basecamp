use super::{Auth, RequestBuilder};
use reqwest::header::HeaderMap;

impl Auth for HeaderMap {
    fn attach(&self, request: RequestBuilder) -> RequestBuilder {
        request.headers(self.clone())
    }
}
