use std::{ffi::OsString, path::PathBuf};

pub type RessourceId = String;

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

impl From<RessourcePath> for RessourcePathComponent {
    fn from(value: RessourcePath) -> Self {
        RessourcePathComponent::Path(value)
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

    pub fn push(&mut self, component: RessourcePathComponent) {
        self.path.push(component);
    }

    pub fn append(&mut self, path: &mut Vec<RessourcePathComponent>) {
        self.path.append(path);
    }

    pub fn append_id(&mut self, path: &mut Vec<RessourceId>) {
        let mut v = Vec::new();
        v.append(path);
        let mut res = v
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<RessourcePathComponent>>();
        self.append(&mut res);
    }

    pub fn up(&mut self) -> Option<RessourcePathComponent> {
        self.path.pop()
    }

    pub fn metadata_path(&self) -> PathBuf {
        let mut path = self.resolve();
        path.add_extension(".meta.json");
        path
    }

    pub fn from_vec(root: PathBuf, path: Vec<RessourcePathComponent>) -> Self {
        RessourcePath { path, root }
    }
}
