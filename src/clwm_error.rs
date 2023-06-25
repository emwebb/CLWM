use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClwmError {
    #[error("the noun type {noun_type:?} could not be found")]
    NounTypeNotFound { noun_type: String },
    #[error("the noun type {noun_type:?} already exists")]
    NounTypeAlreadyExists { noun_type: String },
}
