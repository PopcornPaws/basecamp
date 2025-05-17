use thiserror::Error as ErrorT;

use std::error::Error as StdError;

#[derive(ErrorT, Debug)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] reqwest::Error),
    #[error(transparent)]
    Decode(#[from] serde_json::Error),
    #[error("an unexpecte error occurred: {0}")]
    Unexpected(Box<dyn StdError + Send + Sync>),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Client(this), Self::Client(that)) => this.to_string() == that.to_string(),
            (Self::Decode(this), Self::Decode(that)) => this.to_string() == that.to_string(),
            (Self::Unexpected(this), Self::Unexpected(that)) => {
                this.to_string() == that.to_string()
            }
            _ => false,
        }
    }
}
