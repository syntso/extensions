use std::fmt::{self, Display, Formatter};

use crate::RegistryError;

const LONGEST: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtensionId(String);

impl ExtensionId {
    pub fn parse(text: &str) -> Result<Self, RegistryError> {
        let plain = text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        });
        let starts = text.starts_with(|first: char| first.is_ascii_lowercase());

        if !plain || !starts || text.len() > LONGEST {
            return Err(RegistryError::ExtensionId(text.to_owned()));
        }

        Ok(Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ExtensionId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ExtensionId;

    #[test]
    fn ids_are_plain_lowercase_names() {
        for text in ["docker", "registry-auth", "k8s"] {
            assert!(ExtensionId::parse(text).is_ok(), "{text}");
        }

        for text in [
            "",
            "Docker",
            "dot.ted",
            "-dash",
            "9lives",
            "a/b",
            &"a".repeat(33),
        ] {
            assert!(ExtensionId::parse(text).is_err(), "{text}");
        }
    }
}
