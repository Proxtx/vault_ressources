use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::fs::read_to_string;

#[derive(Debug, Error)]
pub enum RessourceError {
    #[error("IO Error reading metadata for ressource at: {ressource_path}. Error: {error}")]
    MetadataIO {
        error: std::io::Error,
        ressource_path: PathBuf,
    },

    #[error("Malformed metadata for ressource at: {ressource_path}. Error: {error}")]
    MetadataFormat {
        error: serde_json::Error,
        ressource_path: PathBuf,
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
        "Ressource {ressource_type} had an error parsing ressource data of ressource at {ressource_path}. Error: {error}"
    )]
    DataError {
        ressource_type: &'static str,
        ressource_path: PathBuf,
        error: Box<dyn std::error::Error>,
    },
}

pub type RessourceResult<T> = Result<T, RessourceError>;

type RessourceId = String;

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

pub struct RessourcePath {
    pub path: Vec<RessourcePathComponent>,
    pub root: PathBuf,
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
            res.push(std::path::Component::Normal(&OsString::from(id)));
        }
        res
    }

    pub fn append(&mut self, component: RessourcePathComponent) {
        self.path.push(component);
    }

    pub fn up(&mut self) -> Option<RessourcePathComponent> {
        self.path.pop()
    }
}

pub trait RessourceType {
    fn id() -> &'static str;
    fn parse(
        path: &Path,
    ) -> impl Future<Output = Result<Box<Self>, Box<dyn std::error::Error>>> + Send;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RessourceMetadata {
    pub data_extension: String,
    pub type_id: String,
    pub time: DateTime<Utc>,
    pub id: RessourceId,
}

pub struct Ressource<T: RessourceType> {
    pub data: Box<T>,
    pub metadata: RessourceMetadata,
}

impl<T: RessourceType> Ressource<T> {
    pub async fn load(path: RessourcePath) -> RessourceResult<Self> {
        Ressource::load_os_path(&path.resolve()).await
    }

    pub async fn load_os_path(path: &Path) -> RessourceResult<Self> {
        let mut metadata = path.to_path_buf();
        metadata.add_extension("meta.json");

        let metadata: RessourceMetadata =
            serde_json::from_str(&read_to_string(&metadata).await.map_err(|e| {
                RessourceError::MetadataIO {
                    error: e,
                    ressource_path: path.to_path_buf(),
                }
            })?)
            .map_err(|e| RessourceError::MetadataFormat {
                error: e,
                ressource_path: path.to_path_buf(),
            })?;

        if metadata.type_id != T::id() {
            return Err(RessourceError::TypeMismatch {
                ressource_path: path.to_path_buf(),
                expected_type: T::id(),
                ressource_type: metadata.type_id,
            });
        }

        let mut data_path = path.with_added_extension("data");
        data_path.add_extension(metadata.data_extension.clone());

        Ok(Ressource {
            data: T::parse(path)
                .await
                .map_err(|e| RessourceError::DataError {
                    ressource_type: T::id(),
                    ressource_path: path.to_path_buf(),
                    error: e,
                })?,
            metadata,
        })
    }
}

pub struct Ressources {
    pub root: PathBuf,
}

impl Ressources {
    pub async fn load() {}
}
