use std::path::Path;

use serde::Deserialize;

use super::{Capabilities, Contributions};
use crate::error::RegistryError;
use crate::toml_file;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api: String,
    pub description: String,
    pub repository: String,

    #[serde(default)]
    pub capabilities: Capabilities,

    #[serde(default)]
    pub contributes: Contributions,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        toml_file::read(path)
    }
}
