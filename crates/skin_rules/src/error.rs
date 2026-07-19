use std::{io, path::PathBuf};

use skin_core::CoreError;
use thiserror::Error;

pub type RuleResult<T> = Result<T, RuleError>;

#[derive(Debug, Error)]
pub enum RuleError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("output path must be different from input path")]
    OutputEqualsInput,
    #[error("output path must not be inside the input path")]
    OutputInsideInput,
    #[error("output path already exists: {0}")]
    OutputExists(PathBuf),
}

impl RuleError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
