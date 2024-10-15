use super::{Auth, RequestBuilder};
use std::sync::Arc;

/// A cheaply clonable basic auth username and password for API caller authentication.
#[derive(Clone, Debug)]
pub struct Basic {
    pub username: Arc<str>,
    pub password: Arc<str>,
}

impl Basic {
    #[must_use]
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl Auth for Basic {
    fn attach(&self, request: RequestBuilder) -> RequestBuilder {
        request.basic_auth(self.username.as_ref(), Some(self.password.as_ref()))
    }
}
