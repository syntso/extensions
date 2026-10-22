use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::RegistryError;
use crate::pin::Pin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pins(Vec<Pin>);

impl Pins {
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let text = fs::read_to_string(path).map_err(|source| RegistryError::Read {
            path: path.to_owned(),
            source,
        })?;

        Self::parse(&text).map_err(|message| RegistryError::Pins(path.to_owned(), message))
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let value: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
        let entries = value.as_array().ok_or("the pins are a JSON array")?;

        let pins = entries
            .iter()
            .map(Pin::from_value)
            .collect::<Result<Vec<_>, _>>()?;

        let mut seen = BTreeSet::new();

        if let Some(twice) = pins.iter().find(|pin| !seen.insert(pin.id().as_str())) {
            return Err(RegistryError::PinnedTwice {
                id: twice.id().to_string(),
            }
            .to_string());
        }

        Ok(Self(pins))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pin> {
        self.0.iter()
    }

    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.0.iter().map(|pin| pin.id().to_string()).collect();
        ids.sort();

        ids
    }

    pub fn find(&self, id: &str) -> Option<&Pin> {
        self.0.iter().find(|pin| pin.id().as_str() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::Pins;

    const REV: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn the_published_list_reads() {
        assert!(Pins::parse(include_str!("../extensions.json")).is_ok());
    }

    #[test]
    fn an_id_is_pinned_once() {
        let pin = format!(r#"{{ "id": "a", "repository": "https://x.y/a", "rev": "{REV}" }}"#);
        let twice = format!("[{pin}, {pin}]");

        let refusal = Pins::parse(&twice).unwrap_err();

        assert!(refusal.contains("pinned twice"), "{refusal}");
    }

    #[test]
    fn the_list_is_an_array() {
        assert!(Pins::parse("{}").is_err());
        assert!(Pins::parse("[1]").is_err());
    }
}
