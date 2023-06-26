use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClwmError {
    #[error("the provided noun could not be found")]
    NounNotFound,
    #[error("the provided noun type could not be found")]
    NounTypeNotFound,
    #[error("the noun type {noun_type:?} already exists")]
    NounTypeAlreadyExists { noun_type: String },
    #[error("the provided noun has no id")]
    NounHasNoId,
    #[error("the provided noun type has no id")]
    NounTypeHasNoId,
}
