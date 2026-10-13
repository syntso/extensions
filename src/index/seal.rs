use serde::Serialize;

use crate::error::RegistryError;
use crate::manifest::{Capabilities, Manifest};

#[derive(Debug, Serialize)]
pub struct Seal<'a> {
    id: &'a str,
    version: &'a str,
    sha256: &'a str,
    capabilities: &'a Capabilities,
}

impl<'a> Seal<'a> {
    pub fn new(manifest: &'a Manifest, sha256: &'a str) -> Self {
        Self {
            id: &manifest.id,
            version: &manifest.version,
            sha256,
            capabilities: &manifest.capabilities,
        }
    }

    pub fn to_json(&self) -> Result<String, RegistryError> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_one_line_of_json() {
        let manifest: Manifest = toml::from_str(
            "id = \"uptime\"\nname = \"Uptime\"\nversion = \"0.1.0\"\napi = \"0.1\"\n\
             description = \"d\"\nrepository = \"https://example.com\"\n\n\
             [capabilities]\ncall = [\"ssh.exec\"]\n",
        )
        .expect("manifest");

        let json = Seal::new(&manifest, "00ff").to_json().expect("json");

        assert_eq!(
            json,
            r#"{"id":"uptime","version":"0.1.0","sha256":"00ff","capabilities":{"call":["ssh.exec"],"storage":false}}"#
        );
    }
}
