use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{error::RessourceError, path::RessourcePath};

pub trait RessourceType {
    fn id() -> &'static str;
}

pub trait ReadableRessource: RessourceType
where
    Self::Error: 'static,
{
    type Error: std::error::Error;
    fn read(path: &Path) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

impl<T: ReadableRessource + Debug> std::error::Error for ReadableRessourceError<T> {}

#[derive(Debug)]
pub enum ReadableRessourceError<T: ReadableRessource> {
    ReadableRessourceError {
        ressource_type: &'static str,
        readable_error: T::Error,
        ressource_path: RessourcePath,
        path: PathBuf,
    },

    RessourceError(RessourceError),
}

impl<T: ReadableRessource> From<RessourceError> for ReadableRessourceError<T> {
    fn from(value: RessourceError) -> Self {
        Self::RessourceError(value)
    }
}

impl<T: ReadableRessource> From<ReadableRessourceError<T>> for RessourceError {
    fn from(value: ReadableRessourceError<T>) -> Self {
        match value {
            ReadableRessourceError::ReadableRessourceError {
                ressource_type,
                readable_error,
                ressource_path,
                path,
            } => RessourceError::InvalidData {
                ressource_type,
                ressource_path,
                path,
                error: Box::new(readable_error),
            },
            ReadableRessourceError::RessourceError(v) => v,
        }
    }
}

impl<T: ReadableRessource> std::fmt::Display for ReadableRessourceError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", RessourceError::from(self))
    }
}

pub trait WritableRessource: RessourceType
where
    Self::Error: 'static,
{
    type Error: std::error::Error;
    fn data_extension() -> &'static str;
    fn write(&self, path: &Path) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl std::fmt::Display for WriteDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to write ressource data for type {0} for ressource at: {1}. OSPath: {2}. Error: {3}",
            self.ressource_type,
            self.ressource_path,
            self.path.display(),
            self.error
        )
    }
}

#[derive(Debug, Error)]
pub enum WritableRessourceError<T: WritableRessource> {
    #[error(
        "Unable to write ressource data for type {0} for ressource at: {1}. OSPath: {2}. Error: {3}"
    )]
    WriteData {
        ressource_type: &'static str,
        ressource_path: RessourcePath,
        path: PathBuf,
        error: T::Error,
    },

    #[error("{0}")]
    RessourceError(#[from] RessourceError),
}

impl<T: WritableRessource> From<WritableRessourceError<T>> for RessourceError {
    fn from(value: WritableRessourceError<T>) -> Self {
        match value {
            ReadableRessourceError::ReadableRessourceError {
                ressource_type,
                readable_error,
                ressource_path,
                path,
            } => RessourceError::InvalidData {
                ressource_type,
                ressource_path,
                path,
                error: Box::new(readable_error),
            },
            ReadableRessourceError::RessourceError(v) => v,
        }
    }
}
