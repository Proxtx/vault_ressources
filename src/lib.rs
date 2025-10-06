pub mod error;
pub mod folder_ressource;
pub mod path;
pub mod traits;

use crate::error::{RessourceError, RessourceResult, WriteDataError};
use crate::folder_ressource::FolderRessource;
use crate::path::{RessourceId, RessourcePath, RessourcePathComponent};
use crate::traits::{ReadableRessource, RessourceType, WritableRessource};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, path::PathBuf};
use tokio::fs::{self, read_to_string};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RessourceMetadata {
    pub data_extension: String,
    pub type_id: String,
    pub time: DateTime<Utc>,
    pub id: RessourceId,
}

#[derive(Debug)]
pub struct MetaRessource<T: RessourceType> {
    pub metadata: RessourceMetadata,
    pub path: RessourcePath,
    phantom: PhantomData<T>,
}

impl<T: RessourceType> MetaRessource<T> {
    pub async fn load(path: RessourcePath) -> RessourceResult<Self> {
        let metadata_path = path.metadata_path();
        let metadata: RessourceMetadata =
            serde_json::from_str(&read_to_string(&metadata_path).await.map_err(|e| {
                RessourceError::MetadataIO {
                    error: e,
                    path: path.resolve(),
                    ressource_path: path.clone(),
                }
            })?)
            .map_err(|e| RessourceError::MetadataFormat {
                error: e,
                path: path.resolve(),
                ressource_path: path.clone(),
            })?;

        if metadata.type_id != T::id() {
            return Err(RessourceError::TypeMismatch {
                ressource_path: path.resolve().to_path_buf(),
                expected_type: T::id(),
                ressource_type: metadata.type_id,
            });
        }

        Ok(Self {
            metadata,
            path,
            phantom: PhantomData,
        })
    }

    pub fn new(path: RessourcePath) -> RessourceResult<Self>
    where
        T: WritableRessource,
    {
        let id = match path
            .clone()
            .up()
            .ok_or_else(|| RessourceError::RessourceAtRoot {
                path: path.resolve(),
                ressource_path: path.clone(),
            })? {
            RessourcePathComponent::Ressource(id) => id,
            RessourcePathComponent::Path(last_component_path) => {
                return Err(RessourceError::RessourceIdFolded {
                    path: path.resolve(),
                    ressource_path: path,
                    folded: last_component_path,
                });
            }
        };

        let metadata = RessourceMetadata {
            data_extension: T::data_extension().to_string(),
            type_id: T::id().to_string(),
            time: Utc::now(),
            id,
        };

        Ok(Self {
            metadata,
            path,
            phantom: PhantomData,
        })
    }

    pub fn data_path(&self) -> PathBuf {
        let mut path = self.path.resolve();
        path.add_extension("data");
        path.add_extension(self.metadata.data_extension.clone());
        path
    }
}

#[derive(Debug)]
pub struct Ressource<T: RessourceType> {
    pub data: T,
    pub meta: MetaRessource<T>,
}

impl<T: RessourceType> Ressource<T> {
    pub async fn load(path: RessourcePath) -> RessourceResult<Self>
    where
        T: ReadableRessource,
    {
        let meta_ressource = MetaRessource::<T>::load(path.clone()).await?;
        let data = T::read(&meta_ressource.data_path()).await.map_err(|e| {
            RessourceError::InvalidData {
                ressource_type: T::id(),
                path: path.resolve(),
                ressource_path: path.clone(),
                error: Box::new(e),
            }
        })?;

        Ok(Ressource {
            data,
            meta: meta_ressource,
        })
    }

    pub async fn new(path: RessourcePath, data: T) -> RessourceResult<Self>
    where
        T: WritableRessource,
    {
        let meta_ressource = MetaRessource::new(path.clone())?;
        let mut parent_ressource = path.clone();
        parent_ressource
            .up()
            .ok_or_else(|| RessourceError::RessourceAtRoot {
                path: path.resolve(),
                ressource_path: path.clone(),
            })?;

        Ressource::<FolderRessource>::load(path.clone())
            .await
            .map_err(|e| RessourceError::ParentRessource {
                path: path.resolve(),
                ressource_path: path.clone(),
                folder_error: Box::new(e),
            })?;

        fs::write(
            path.metadata_path(),
            serde_json::to_string(&meta_ressource.metadata).unwrap(),
        )
        .await
        .map_err(|e| RessourceError::WriteMetadataIO {
            error: e,
            ressource_path: path.clone(),
            path: path.resolve(),
        })?;

        let data_path = meta_ressource.data_path();
        if let Err(write_data_error) = data.write(&data_path).await.map_err(|e| WriteDataError {
            ressource_type: T::id(),
            ressource_path: path.clone(),
            path: path.resolve(),
            error: Box::new(e),
        }) {
            match fs::remove_file(path.metadata_path()).await {
                Ok(_) => return Err(RessourceError::WriteDataError(write_data_error)),
                Err(e) => {
                    return Err(RessourceError::DeleteMetadataError {
                        data_error: write_data_error,
                        error: e,
                    });
                }
            }
        }

        Ok(Ressource {
            data,
            meta: meta_ressource,
        })
    }
}

pub struct Ressources {
    pub root: PathBuf,
}

impl Ressources {
    pub async fn load() {}
}
