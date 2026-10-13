use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

#[derive(Debug, Clone)]
pub struct Extension {
    folder: PathBuf,
    manifest: Manifest,
}

impl Extension {
    pub fn new(folder: PathBuf, manifest: Manifest) -> Self {
        Self { folder, manifest }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}
