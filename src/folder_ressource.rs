use crate::traits::{ReadableRessource, RessourceType, WritableRessource};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum FolderRessourceError {
    #[error("FolderRessource: IO Error checking for folder at {path}. Error: {error}")]
    CheckingForFolder {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("FolderRessource: Not a folder at {path}")]
    NotAFolder { path: PathBuf },

    #[error("FolderRessource: Unable to create folder at: {path}. Error: {error}")]
    CreatingFolder {
        path: PathBuf,
        error: std::io::Error,
    },
}

pub struct FolderRessource {}

impl RessourceType for FolderRessource {
    fn id() -> &'static str {
        "core/folder"
    }
}

impl ReadableRessource for FolderRessource {
    type Error = FolderRessourceError;
    async fn read(path: &Path) -> Result<Self, FolderRessourceError> {
        if !fs::File::open(path)
            .await
            .map_err(|e| FolderRessourceError::CheckingForFolder {
                path: path.to_path_buf(),
                error: e,
            })?
            .metadata()
            .await
            .map_err(|e| FolderRessourceError::CheckingForFolder {
                path: path.to_path_buf(),
                error: e,
            })?
            .is_dir()
        {
            return Err(FolderRessourceError::NotAFolder {
                path: path.to_path_buf(),
            });
        }
        Ok(Self {})
    }
}

impl WritableRessource for FolderRessource {
    type Error = FolderRessourceError;
    async fn write(&self, path: &Path) -> Result<(), FolderRessourceError> {
        fs::create_dir(path)
            .await
            .map_err(|e| FolderRessourceError::CreatingFolder {
                path: path.to_path_buf(),
                error: e,
            })
    }

    fn data_extension() -> &'static str {
        ""
    }
}
