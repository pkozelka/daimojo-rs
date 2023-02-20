use std::ffi::NulError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, MojoError>;

#[derive(ThisError,Debug)]
pub enum MojoError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("CSV Error")]
    CsvError(#[from] csv::Error),
    #[error("DlOpen Error")]
    DlOpenError(#[from] dlopen2::Error),
    #[error("Pipeline model is not valid - check your license settings")]
    InvalidModel,
    #[error("Cannot create pipeline")]
    InvalidPipeline,
    #[error("Null Error")]
    NulError(#[from] NulError),
    #[error("Missing input column: {0}")]
    MissingInputColumn(usize),
    #[error("invalid index of input column: {0}")]
    InvalidInputIndex(usize),
    #[error("invalid index of output column: {0}")]
    InvalidOutputIndex(usize),
    #[error("{0}: Not a supported API inside version '{1}'")]
    UnsupportedApi(String, String)
}
