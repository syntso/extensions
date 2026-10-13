use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::error::RegistryError;

const LIFETIME: Duration = Duration::from_hours(45 * 24);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct IndexSeal {
    serial: u64,
    expires: u64,
}

impl IndexSeal {
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            serial: now,
            expires: now + LIFETIME.as_secs(),
        }
    }

    pub fn to_json(self) -> Result<String, RegistryError> {
        Ok(serde_json::to_string(&self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expires_well_after_the_serial() {
        let seal = IndexSeal::now();

        assert!(seal.expires > seal.serial + 30 * 24 * 3600);
        assert!(seal.to_json().expect("json").starts_with(r#"{"serial":"#));
    }
}
