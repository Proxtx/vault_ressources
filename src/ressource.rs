use crate::error::{RessourceError, RessourceResult, WriteDataError};
use crate::folder_ressource::FolderRessource;
use crate::meta::MetaRessource;
use crate::path::RessourcePath;
use crate::traits::{ReadableRessource, RessourceType, WritableRessource};
use tokio::fs;

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
