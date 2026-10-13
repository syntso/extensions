use serde::Serialize;

use super::Listing;
use crate::error::RegistryError;

const FORMAT: u32 = 1;

#[derive(Debug, Serialize)]
pub struct Index {
    format: u32,
    extensions: Vec<Listing>,
}

impl Index {
    pub fn new(extensions: Vec<Listing>) -> Self {
        Self {
            format: FORMAT,
            extensions,
        }
    }

    pub fn to_json(&self) -> Result<String, RegistryError> {
        let json = serde_json::to_string_pretty(self)?;

        Ok(format!("{json}\n"))
    }
}
