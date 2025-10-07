use std::{ffi::OsString, path::PathBuf};

pub type RessourceId = String;

#[derive(Debug, Clone)]
pub struct RessourcePath {
    pub path: Vec<RessourceId>,
    pub root: PathBuf,
}

impl std::fmt::Display for RessourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join("/"))
    }
}

impl RessourcePath {
    pub fn resolve(&self) -> PathBuf {
        let mut res = self.root.clone();
        for (i, id) in self.path.iter().enumerate() {
            let cmppath = if i == self.path.len() - 1 {
                id.clone()
            } else {
                format!("{id}.data")
            };
            res.push(std::path::Component::Normal(&OsString::from(cmppath)));
        }
        res
    }

    pub fn push(&mut self, component: impl Into<RessourceId>) {
        self.path.push(component.into());
    }

    pub fn with_child(&self, component: impl Into<RessourceId>) -> RessourcePath {
        let mut other = self.clone();
        other.push(component.into());
        other
    }

    pub fn append(&mut self, path: &mut Vec<RessourceId>) {
        self.path.append(path);
    }

    pub fn with_children(&self, path: &mut Vec<RessourceId>) -> RessourcePath {
        let mut other = self.clone();
        other.append(path);
        other
    }

    pub fn up(&mut self) -> Option<RessourceId> {
        self.path.pop()
    }

    pub fn with_parent(&self) -> Option<RessourcePath> {
        let mut other = self.clone();
        other.up().map(|_v| other)
    }

    pub fn metadata_path(&self) -> PathBuf {
        let mut path = self.resolve();
        path.add_extension("meta.json");
        path
    }

    pub fn from_vec(root: PathBuf, path: Vec<RessourceId>) -> Self {
        RessourcePath { path, root }
    }

    pub fn new(root: PathBuf) -> Self {
        RessourcePath {
            path: Vec::new(),
            root,
        }
    }
}
