use crate::path::RessourcePath;
use std::{fmt::Debug, path::PathBuf};
use thiserror::Error;

#[derive(Debug)]
pub struct WriteDataError {
    pub ressource_type: &'static str,
    pub ressource_path: RessourcePath,
    pub path: PathBuf,
    pub error: Box<dyn std::error::Error>,
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

impl std::error::Error for WriteDataError {}

#[derive(Debug, Error)]
pub enum RessourceError {
    #[error(
        "IO Error reading metadata for ressource at: {ressource_path}. OSPath: {path}. Error: {error}"
    )]
    MetadataIO {
        error: std::io::Error,
        ressource_path: RessourcePath,
        path: PathBuf,
    },

    #[error(
        "IO Error writing metadata for ressource at {ressource_path}. OSPath: {path}. Error: {error}"
    )]
    WriteMetadataIO {
        error: std::io::Error,
        ressource_path: RessourcePath,
        path: PathBuf,
    },

    #[error(
        "Malformed metadata for ressource at: {ressource_path}. OSPath: {path}. Error: {error}"
    )]
    MetadataFormat {
        error: serde_json::Error,
        ressource_path: RessourcePath,
        path: PathBuf,
    },

    #[error(
        "Ressource type mismatch. Ressource at {ressource_path} hat type {ressource_type} but was loaded with {expected_type}"
    )]
    TypeMismatch {
        ressource_path: PathBuf,
        ressource_type: String,
        expected_type: &'static str,
    },

    #[error(
        "Ressource {ressource_type} had an error parsing ressource data of ressource at {ressource_path}. OSPath: {path}. Error: {error}"
    )]
    InvalidData {
        ressource_type: &'static str,
        ressource_path: RessourcePath,
        path: PathBuf,
        error: Box<dyn std::error::Error>,
    },

    #[error("{0}")]
    WriteDataError(#[from] WriteDataError),

    #[error(
        "Was unable to delete metadata file after beeing unable to write data: {data_error}. Deletion Error: {error}"
    )]
    DeleteMetadataError {
        data_error: WriteDataError,
        error: std::io::Error,
    },

    #[error("Can't create ressource at /. RessourcePath: {ressource_path}. OSPath: {path}")]
    RessourceAtRoot {
        path: PathBuf,
        ressource_path: RessourcePath,
    },

    #[error(
        "Can't create ressource with folded Id: RessourcePath: {ressource_path}. OSPath: {path}. Folded: {folded}"
    )]
    RessourceIdFolded {
        path: PathBuf,
        ressource_path: RessourcePath,
        folded: RessourcePath,
    },

    #[error(
        "Can't create ressource. Parent ressource can't be loaded: RessourcesPath: {ressource_path}. OSPath: {path}. FolderRessourceError: {folder_error}"
    )]
    ParentRessource {
        path: PathBuf,
        ressource_path: RessourcePath,
        folder_error: Box<RessourceError>,
    },
}

pub type RessourceResult<T> = Result<T, RessourceError>;
