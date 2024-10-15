use super::{Auth, RequestBuilder};
use std::sync::Arc;

/// A cheaply clonable login form data with username and password.
#[derive(Clone, Debug)]
pub struct Login {
    pub username: Arc<str>,
    pub password: Arc<str>,
}

impl Login {
    #[must_use]
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl Auth for Login {
    fn attach(&self, request: RequestBuilder) -> RequestBuilder {
        let params: std::collections::HashMap<&str, &str> = [
            ("username", self.username.as_ref()),
            ("password", self.password.as_ref()),
        ]
        .into_iter()
        .collect();
        request.form(&params)
    }
}
