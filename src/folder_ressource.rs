use crate::{
    path::RessourceId,
    traits::{ReadableRessource, RessourceType, WritableRessource},
};
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

    #[error("FolderRessource: Unable to read next folder entry at {path}. Error: {error}")]
    NextEntry {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("FolderRessource: Unable to create folder at: {path}. Error: {error}")]
    CreatingFolder {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error(
        "FolderRessource: Invalid Filename in folder at: {path}. Invalid filename display: {filename}"
    )]
    Filename { path: PathBuf, filename: String },
}

pub struct FolderRessource {
    pub ressources: Vec<RessourceId>,
}

impl RessourceType for FolderRessource {
    fn id() -> &'static str {
        "core/folder"
    }
}

impl ReadableRessource for FolderRessource {
    type Error = FolderRessourceError;
    async fn read(path: &Path) -> Result<Self, FolderRessourceError> {
        let mut stream =
            fs::read_dir(path)
                .await
                .map_err(|e| FolderRessourceError::CheckingForFolder {
                    path: path.to_path_buf(),
                    error: e,
                })?;

        let mut ressources = Vec::new();

        while let Some(entry) =
            stream
                .next_entry()
                .await
                .map_err(|e| FolderRessourceError::NextEntry {
                    path: path.to_path_buf(),
                    error: e,
                })?
        {
            let filename =
                entry
                    .file_name()
                    .into_string()
                    .map_err(|e| FolderRessourceError::Filename {
                        path: path.to_path_buf(),
                        filename: format!("{}", e.display()),
                    })?;

            if let Some(ressource_id) = filename.strip_suffix(".meta.json") {
                ressources.push(ressource_id.to_string());
            }
        }

        Ok(Self { ressources })
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
