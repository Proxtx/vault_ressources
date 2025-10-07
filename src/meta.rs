use std::{marker::PhantomData, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;

use crate::{
    error::{RessourceError, RessourceResult},
    path::{RessourceId, RessourcePath, RessourcePathComponent},
    traits::{RessourceType, WritableRessource},
};

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
