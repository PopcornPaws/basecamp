use super::{Auth, RequestBuilder};
use std::sync::Arc;

/// A cheaply clonable bearer token used for API caller authentication.
#[derive(Clone, Debug)]
pub struct Bearer {
    pub token: Arc<str>,
}

impl Bearer {
    #[must_use]
    pub fn new(token: &str) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl Auth for Bearer {
    fn attach(&self, request: RequestBuilder) -> RequestBuilder {
        request.bearer_auth(self.token.as_ref())
    }
}
