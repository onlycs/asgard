use std::{
    fs::File,
    io,
    panic::Location,
    sync::{MutexGuard, PoisonError},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateLoggerError {
    #[error("At {location}: IO error: {error}")]
    IO {
        #[from]
        error: io::Error,
        location: &'static Location<'static>,
    },

    #[error("At {location}: Failed to set logger: {error}")]
    SetLogger {
        #[from]
        error: log::SetLoggerError,
        location: &'static Location<'static>,
    },
}

#[derive(Error, Debug)]
pub(crate) enum WriteFileError<'a> {
    #[error("At {location}: IO error: {error}")]
    IO {
        #[from]
        error: io::Error,
        location: &'static Location<'static>,
    },

    #[error("At {location}: Failed to lock file: {error}")]
    Lock {
        error: PoisonError<MutexGuard<'a, File>>,
        location: &'static Location<'static>,
    },
}

impl<'a> From<PoisonError<MutexGuard<'a, File>>> for WriteFileError<'a> {
    #[track_caller]
    fn from(error: PoisonError<MutexGuard<'a, File>>) -> Self {
        WriteFileError::Lock {
            error,
            location: Location::caller(),
        }
    }
}
