use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    marker::PhantomData,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::fs::{self, read_to_string};

#[derive(Debug)]
pub struct WriteDataError {
    ressource_type: &'static str,
    ressource_path: RessourcePath,
    path: PathBuf,
    error: Box<dyn std::error::Error>,
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

type RessourceId = String;

#[derive(Debug, Clone)]
pub enum RessourcePathComponent {
    Ressource(RessourceId),
    Path(RessourcePath),
}

impl RessourcePathComponent {
    pub fn resolve(&self) -> Vec<&RessourceId> {
        match &self {
            RessourcePathComponent::Ressource(id) => vec![id],
            RessourcePathComponent::Path(path) => path.resolve_components(),
        }
    }
}

impl From<RessourceId> for RessourcePathComponent {
    fn from(value: RessourceId) -> Self {
        RessourcePathComponent::Ressource(value)
    }
}

#[derive(Debug, Clone)]
pub struct RessourcePath {
    pub path: Vec<RessourcePathComponent>,
    pub root: PathBuf,
}

impl std::fmt::Display for RessourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.resolve_components()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}

impl RessourcePath {
    pub fn resolve_components(&self) -> Vec<&RessourceId> {
        let mut res = Vec::new();
        for compnent in &self.path {
            let mut other = compnent.resolve();
            res.append(&mut other);
        }
        res
    }

    pub fn resolve_into_ressource_path(&self) -> RessourcePath {
        let components = self
            .resolve_components()
            .into_iter()
            .map(|v| RessourcePathComponent::Ressource(v.clone()))
            .collect();
        RessourcePath {
            path: components,
            root: self.root.clone(),
        }
    }

    pub fn resolve(&self) -> PathBuf {
        let mut res = self.root.clone();
        let ids = self.resolve_components();
        for id in ids {
            res.push(std::path::Component::Normal(&OsString::from(format!(
                "{id}.data"
            ))));
        }
        res
    }

    pub fn append(&mut self, component: RessourcePathComponent) {
        self.path.push(component);
    }

    pub fn up(&mut self) -> Option<RessourcePathComponent> {
        self.path.pop()
    }

    pub fn metadata_path(&self) -> PathBuf {
        let mut path = self.resolve();
        path.add_extension(".meta.json");
        path
    }
}

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

#[derive(Debug, Error)]
pub enum ReadableRessourceError<T: ReadableRessource> {
    #[error(
        "Ressource {ressource_type} had an error parsing ressource data of ressource at {ressource_path}. OSPath: {path}. Error: {readable_error}"
    )]
    ReadableRessourceError {
        ressource_type: &'static str,
        readable_error: T::Error,
        ressource_path: RessourcePath,
        path: PathBuf,
    },

    #[error("{0}")]
    RessourceError(#[from] RessourceError),
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

pub trait WritableRessource: RessourceType
where
    Self::Error: 'static,
{
    type Error: std::error::Error;
    fn data_extension() -> &'static str;
    fn write(&self, path: &Path) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

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
