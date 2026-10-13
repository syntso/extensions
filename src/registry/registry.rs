use std::path::Path;

use serde::Deserialize;

use super::Entry;
use crate::error::RegistryError;
use crate::toml_file;

const FILE: &str = "extensions.toml";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    #[serde(rename = "extension", default)]
    entries: Vec<Entry>,
}

impl Registry {
    pub fn load(root: &Path) -> Result<Self, RegistryError> {
        toml_file::read(&root.join(FILE))
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}
