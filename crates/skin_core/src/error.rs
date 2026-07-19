use std::{io, path::PathBuf};

use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("input path does not exist: {0}")]
    MissingInput(PathBuf),
    #[error("input path is not a directory: {0}")]
    NotDirectory(PathBuf),
}

impl CoreError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
