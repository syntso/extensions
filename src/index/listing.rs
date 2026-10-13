use serde::Serialize;

use crate::manifest::{Capabilities, Manifest};

#[derive(Debug, Serialize)]
pub struct Listing {
    id: String,
    name: String,
    version: String,
    api: String,
    description: String,
    repository: String,
    commit: String,
    file: String,
    sha256: String,
    signature: String,
    capabilities: Capabilities,
}

impl Listing {
    pub fn new(
        manifest: &Manifest,
        commit: String,
        file: String,
        sha256: String,
        signature: String,
    ) -> Self {
        Self {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            api: manifest.api.clone(),
            description: manifest.description.clone(),
            repository: manifest.repository.clone(),
            commit,
            file,
            sha256,
            signature,
            capabilities: manifest.capabilities.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::index::Index;

    #[test]
    fn writes_every_field() {
        let manifest: Manifest = toml::from_str(
            "id = \"uptime\"\nname = \"Uptime\"\nversion = \"0.1.0\"\napi = \"0.1\"\n\
             description = \"Shows uptime.\"\nrepository = \"https://github.com/syntso/extension-uptime\"\n\n\
             [capabilities]\ncall = [\"ssh.exec\"]\n",
        )
        .expect("manifest");

        let listing = Listing::new(
            &manifest,
            "abc".to_owned(),
            "uptime.wasm".to_owned(),
            "00ff".to_owned(),
            "sig".to_owned(),
        );

        let json: Value = serde_json::from_str(&Index::new(vec![listing]).to_json().expect("json"))
            .expect("valid json");

        assert_eq!(
            json,
            json!({
                "format": 1,
                "extensions": [{
                    "id": "uptime",
                    "name": "Uptime",
                    "version": "0.1.0",
                    "api": "0.1",
                    "description": "Shows uptime.",
                    "repository": "https://github.com/syntso/extension-uptime",
                    "commit": "abc",
                    "file": "uptime.wasm",
                    "sha256": "00ff",
                    "signature": "sig",
                    "capabilities": { "call": ["ssh.exec"], "storage": false }
                }]
            })
        );
    }
}
