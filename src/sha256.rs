use std::fmt::{self, Display, Formatter};

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256 as Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256(String);

impl Sha256 {
    pub fn of(bytes: &[u8]) -> Self {
        Self(HEXLOWER.encode(&Hasher::digest(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Sha256 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Sha256;

    #[test]
    fn a_digest_is_lowercase_hex() {
        assert_eq!(
            Sha256::of(b"hello").as_str(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
