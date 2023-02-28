use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdtError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error("Other")]
    Other,
}
