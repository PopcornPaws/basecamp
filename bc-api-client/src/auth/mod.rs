mod basic;
mod bearer;
mod headers;
mod login;

pub use basic::Basic;
pub use bearer::Bearer;
pub use login::Login;

use reqwest::RequestBuilder;

/// Defines an authentication method that can be attached to a http request.
pub trait Auth {
    fn attach(&self, request: RequestBuilder) -> RequestBuilder;
}

impl Auth for () {
    fn attach(&self, request: RequestBuilder) -> RequestBuilder {
        request
    }
}
