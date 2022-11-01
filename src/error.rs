use std::ffi::NulError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, MojoError>;

#[derive(ThisError,Debug)]
pub enum MojoError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Pipeline is not valid - check your license settings")]
    InvalidPipeline,
    #[error("Null Error")]
    NulError(#[from] NulError),
}
