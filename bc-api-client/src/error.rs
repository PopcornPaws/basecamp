use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GenericError {
    pub error: String,
    pub message: String,
}

impl GenericError {
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn new<E: ToString>(error: E) -> Self {
        Self {
            error: error.to_string(),
            message: String::new(),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn with_message<M: ToString>(mut self, message: M) -> Self {
        self.message = message.to_string();
        self
    }
}
