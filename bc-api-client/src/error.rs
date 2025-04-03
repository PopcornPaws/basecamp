use thiserror::Error as ErrorT;

#[derive(ErrorT, Debug)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Client(this), Self::Client(that)) => this.to_string() == that.to_string(),
            (Self::Serde(this), Self::Serde(that)) => this.to_string() == that.to_string(),
            _ => false,
        }
    }
}
