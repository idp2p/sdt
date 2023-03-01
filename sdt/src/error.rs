use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdtError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Other")]
    Other,
}
