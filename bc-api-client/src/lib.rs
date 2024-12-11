#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

/// Various authentication method implementations for interacting with APIs.
pub mod auth;
pub mod request;
pub mod response;

use auth::Auth;
use request::Request;

pub use reqwest;
use reqwest::{Client, Method};

use std::marker::PhantomData;
use std::sync::Arc;

#[must_use]
pub struct ApiClientBuilder<'a> {
    client: Client,
    base_url: &'a str,
    auth: Box<dyn Auth + Send + Sync>,
}

impl<'a> ApiClientBuilder<'a> {
    pub fn new(base_url: &'a str) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth: Box::new(()),
        }
    }

    pub fn with_auth<A: Auth + Send + Sync + 'static>(self, auth: A) -> Self {
        Self {
            client: self.client,
            base_url: self.base_url,
            auth: Box::new(auth),
        }
    }

    #[must_use]
    pub fn build<T>(self) -> ApiClient<T> {
        ApiClient {
            client: self.client,
            base_url: Arc::from(self.base_url),
            auth: Arc::from(self.auth),
            _api: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ApiClient<T> {
    /// The client to dispatch calls with.
    pub client: Client,
    /// The base url where the API can be reached.
    pub base_url: Arc<str>,
    /// The authentication method attached to this client.
    ///
    /// Once initialized, it can still be overridden if the same API needs a different
    /// authentication method.
    pub auth: Arc<dyn Auth + Send + Sync>,
    _api: PhantomData<T>,
}

impl<T> ApiClient<T> {
    /// Initializes a new `ApiClient` with the API base url and the required auth method.
    ///
    /// # Examples
    /// ```
    /// # use bc_api_client::ApiClient;
    /// # type MyApi = ();
    /// // Instead of passing a new client, it is recommended to clone an existing
    /// // reqwest::Client.
    /// // Note, that () implements the Auth trait, and it means that no
    /// // authentication method is applied to the dispatched requests.
    /// let client = ApiClient::<MyApi>::new(reqwest::Client::new(), "example.com", ());
    /// ```
    pub fn new<A: Auth + Send + Sync + 'static>(client: Client, base_url: &str, auth: A) -> Self {
        Self {
            client,
            base_url: Arc::from(base_url),
            auth: Arc::new(auth),
            _api: PhantomData,
        }
    }

    /// Sets the authentication method applied to API calls.
    ///
    /// Essentially clones the api client, which is cheap due to
    /// the underlying Arc pointers.
    ///
    /// # Examples
    /// ```
    /// # use bc_api_client::ApiClient;
    /// # use bc_api_client::auth::Bearer;
    /// # type MyApi = ();
    /// let reqwest_client = reqwest::Client::new();
    /// let client = ApiClient::<MyApi>::new(reqwest_client, "example.com", ());
    /// let auth = Bearer::new("my-bearer-token");
    /// let auth_client = client.with_auth_cloned(auth);
    /// ```
    #[must_use]
    pub fn with_auth_cloned<A: Auth + Send + Sync + 'static>(&self, auth: A) -> Self {
        Self {
            client: self.client.clone(), // cheap due to Arc (and recommended)
            base_url: Arc::clone(&self.base_url),
            auth: Arc::new(auth),
            _api: PhantomData,
        }
    }

    #[must_use]
    pub fn request(self, method: Method, route: &str) -> Request {
        let url = format!("{}{}", self.base_url, route);
        let request = self.client.request(method, url);
        self.auth.attach(request).into()
    }

    #[must_use]
    pub fn get(self, route: &str) -> Request {
        self.request(Method::GET, route)
    }

    #[must_use]
    pub fn post(self, route: &str) -> Request {
        self.request(Method::POST, route)
    }

    #[must_use]
    pub fn put(self, route: &str) -> Request {
        self.request(Method::PUT, route)
    }

    #[must_use]
    pub fn patch(self, route: &str) -> Request {
        self.request(Method::PATCH, route)
    }

    #[must_use]
    pub fn delete(self, route: &str) -> Request {
        self.request(Method::DELETE, route)
    }

    #[must_use]
    pub fn head(self, route: &str) -> Request {
        self.request(Method::HEAD, route)
    }
}
