use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use crate::RegistryError;
use crate::listing::Listing;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Index {
    listings: Vec<Listing>,
}

impl Index {
    pub fn new(listings: Vec<Listing>) -> Self {
        Self { listings }
    }

    pub fn read(path: &Path) -> Result<Self, RegistryError> {
        let label = path.display().to_string();
        let text = fs::read_to_string(path).map_err(|source| RegistryError::Read {
            path: path.to_owned(),
            source,
        })?;

        let value: Value = serde_json::from_str(&text).map_err(|error| RegistryError::Json {
            path: label.clone(),
            message: error.to_string(),
        })?;

        let entries = value
            .get("extensions")
            .and_then(Value::as_array)
            .ok_or(RegistryError::ListingField(label, "extensions"))?;

        let listings = entries
            .iter()
            .map(Listing::from_value)
            .collect::<Result<_, _>>()?;

        Ok(Self { listings })
    }

    pub fn listings(&self) -> &[Listing] {
        &self.listings
    }

    pub fn into_listings(self) -> Vec<Listing> {
        self.listings
    }

    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .listings
            .iter()
            .map(|listing| listing.manifest().id().to_string())
            .collect();
        ids.sort();

        ids
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let extensions: Vec<Value> = self.listings.iter().map(Listing::to_value).collect();
        let value = json!({ "extensions": extensions });
        let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());

        format!("{text}\n").into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::Index;

    #[test]
    fn an_empty_index_lists_nothing() {
        let bytes = Index::default().to_bytes();

        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            "{\n  \"extensions\": []\n}\n"
        );
    }
}
