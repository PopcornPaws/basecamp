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
