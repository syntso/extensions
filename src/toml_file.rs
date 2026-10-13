use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::RegistryError;

pub fn read<T: DeserializeOwned>(path: &Path) -> Result<T, RegistryError> {
    let text = fs::read_to_string(path).map_err(|source| RegistryError::Read {
        path: path.to_owned(),
        source,
    })?;

    toml::from_str(&text).map_err(|source| RegistryError::Parse {
        path: path.to_owned(),
        source,
    })
}
