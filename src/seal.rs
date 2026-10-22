use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::RegistryError;

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seal {
    serial: u64,
    expires: u64,
}

impl Seal {
    pub const DAYS: u64 = 21;

    pub fn new(serial: Option<u64>, days: u64) -> Result<Self, RegistryError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| RegistryError::Clock)?
            .as_secs();

        Ok(Self {
            serial: serial.unwrap_or(now),
            expires: now + days * SECONDS_PER_DAY,
        })
    }

    pub fn serial(self) -> u64 {
        self.serial
    }

    pub fn expires(self) -> u64 {
        self.expires
    }

    pub fn trusted_comment(self) -> String {
        json!({ "serial": self.serial, "expires": self.expires }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Seal;

    #[test]
    fn the_serial_defaults_to_the_time_and_the_seal_expires_later() {
        let seal = Seal::new(None, 21).unwrap();

        assert_eq!(seal.expires() - seal.serial(), 21 * 24 * 60 * 60);
    }

    #[test]
    fn the_comment_is_what_the_core_reads() {
        let seal = Seal::new(Some(7), 1).unwrap();
        let comment: serde_json::Value = serde_json::from_str(&seal.trusted_comment()).unwrap();

        assert_eq!(comment["serial"], 7);
        assert_eq!(comment["expires"], seal.expires());
    }
}
