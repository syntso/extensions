use std::fmt::{self, Display, Formatter};

use crate::RegistryError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository(String);

impl Repository {
    pub fn parse(text: &str) -> Result<Self, RegistryError> {
        let web = text
            .strip_prefix("https://")
            .is_some_and(|rest| !rest.is_empty());
        let local = text.starts_with('/');
        let plain = !text
            .chars()
            .any(|character| character.is_whitespace() || character.is_control());

        if !(web || local) || !plain {
            return Err(RegistryError::Repository(text.to_owned()));
        }

        Ok(Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Repository {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Repository;

    #[test]
    fn repositories_are_https_urls_or_absolute_paths() {
        for text in ["https://github.com/someone/synt-docker", "/tmp/docker"] {
            assert!(Repository::parse(text).is_ok(), "{text}");
        }

        for text in [
            "",
            "--upload-pack=x",
            "git@github.com:a/b",
            "http://x",
            "https://",
            "relative/path",
            "https://a b",
        ] {
            assert!(Repository::parse(text).is_err(), "{text}");
        }
    }
}
