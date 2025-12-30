use jbonsai::{EngineError, model::interporation_weight::WeightError};
use jpreprocess::error::JPreprocessError;

#[derive(Debug, thiserror::Error)]
pub enum SyrinxError {
  #[error("jpreprocess failed: {0}")]
  JPreprocess(#[from] JPreprocessError),
  #[error("jbonsai failed: {0}")]
  Engine(#[from] EngineError),
  #[error("synthesis option invalid: {0}")]
  Weight(#[from] WeightError),
  #[error("opus2 failed")]
  Opus2(#[from] opus2::Error),
  #[error("lock failed")]
  LockFailed,
}

impl From<SyrinxError> for napi::Error {
  fn from(value: SyrinxError) -> Self {
    match value {
      SyrinxError::JPreprocess(JPreprocessError::Io(_)) => {
        napi::Error::new(napi::Status::GenericFailure, "IO error")
      }
      SyrinxError::JPreprocess(err) => napi::Error::new(napi::Status::GenericFailure, err),
      SyrinxError::Engine(EngineError::ModelError(err)) => {
        napi::Error::new(napi::Status::InvalidArg, err)
      }
      SyrinxError::Engine(err) => napi::Error::new(napi::Status::GenericFailure, err),
      SyrinxError::Weight(err) => napi::Error::new(napi::Status::InvalidArg, err),
      SyrinxError::Opus2(err) => napi::Error::new(napi::Status::GenericFailure, err),
      e => napi::Error::new(napi::Status::GenericFailure, e),
    }
  }
}

pub type SyrinxResult<T> = Result<T, SyrinxError>;
