use std::fmt::{self, Display, Formatter};

use crate::RegistryError;

const LENGTH: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rev(String);

impl Rev {
    pub fn parse(text: &str) -> Result<Self, RegistryError> {
        let hex = text
            .chars()
            .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character));

        if text.len() != LENGTH || !hex {
            return Err(RegistryError::Rev(text.to_owned()));
        }

        Ok(Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Rev {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Rev;

    #[test]
    fn only_full_lowercase_commits_pass() {
        assert!(Rev::parse(&"a".repeat(40)).is_ok());

        for text in [
            "",
            "main",
            "v1.0.0",
            &"a".repeat(39),
            &"A".repeat(40),
            &"g".repeat(40),
        ] {
            assert!(Rev::parse(text).is_err(), "{text}");
        }
    }
}
